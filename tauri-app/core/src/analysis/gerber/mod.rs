use crate::domain::ConvertError;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const MAX_GERBER_SIZE: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberMetadata {
    pub cad_program: Option<String>,
    pub creation_date: Option<String>,
    pub layer_type: Option<String>,
    pub side: Option<String>,
    pub units: Option<String>,
    pub coordinate_format: Option<GerberCoordinateFormat>,
    pub statistics: GerberStatistics,
    pub apertures: Vec<GerberAperture>,
    pub d_code_validation: GerberDCodeValidation,
    pub structure_validation: GerberStructureValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberCoordinateFormat {
    pub x_integer_digits: u8,
    pub x_decimal_digits: u8,
    pub y_integer_digits: u8,
    pub y_decimal_digits: u8,
    pub notation: Option<String>,
    pub zero_suppression: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberStatistics {
    pub line_count: u64,
    pub arc_count: u64,
    pub flash_count: u64,
    pub region_count: u64,
    pub aperture_count: u64,
    pub aperture_macro_count: u64,
    pub used_d_codes: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberAperture {
    pub d_code: u32,
    pub template: String,
    pub shape: String,
    pub modifiers: Vec<String>,
    pub has_hole: Option<bool>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberDCodeValidation {
    pub undefined_used: Vec<u32>,
    pub unused_defined: Vec<u32>,
    pub duplicate_definitions: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberStructureValidation {
    pub is_valid: bool,
    pub issues: Vec<GerberStructureIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GerberStructureIssue {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub command: Option<String>,
}

pub fn extract_metadata(path: &Path) -> Result<GerberMetadata, ConvertError> {
    let file_metadata = fs::metadata(path)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", path.display())))?;
    if !file_metadata.is_file() {
        return Err(ConvertError::InvalidInput(
            "Ожидается локальный Gerber-файл".into(),
        ));
    }
    if file_metadata.len() > MAX_GERBER_SIZE {
        return Err(ConvertError::InvalidInput(
            "Gerber-файл превышает допустимый размер 32 МБ".into(),
        ));
    }

    let content = fs::read_to_string(path).map_err(|error| {
        ConvertError::InvalidInput(format!(
            "Gerber должен быть текстовым файлом в UTF-8 или ASCII: {error}"
        ))
    })?;
    parse_metadata(&content)
}

fn parse_metadata(content: &str) -> Result<GerberMetadata, ConvertError> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    if !content.contains("%TF.") && !content.contains("%FS") && !content.contains("M02*") {
        return Err(ConvertError::InvalidInput("Файл не похож на Gerber".into()));
    }

    let generation = attribute_fields(content, "GenerationSoftware");
    let cad_program = generation.and_then(|fields| {
        let value = fields
            .into_iter()
            .filter(|field| !field.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        (!value.is_empty()).then_some(value)
    });
    let creation_date = attribute_fields(content, "CreationDate")
        .and_then(|fields| fields.into_iter().find(|field| !field.is_empty()));
    let file_function = attribute_fields(content, "FileFunction");
    let layer_type = file_function
        .as_ref()
        .and_then(|fields| fields.first())
        .filter(|value| !value.is_empty())
        .cloned();
    let side = file_function.as_ref().and_then(|fields| {
        fields
            .iter()
            .find_map(|field| match field.to_ascii_lowercase().as_str() {
                "top" => Some("top".to_owned()),
                "bot" | "bottom" => Some("bottom".to_owned()),
                "inr" | "inner" => Some("inner".to_owned()),
                _ => None,
            })
    });
    let units = extended_command(content, "MO").and_then(|value| {
        if value.starts_with("MM") {
            Some("mm".to_owned())
        } else if value.starts_with("IN") {
            Some("inch".to_owned())
        } else {
            None
        }
    });
    let coordinate_format = parse_coordinate_format(content);
    let analysis = analyze_graphics(content);
    let structure_validation = validate_structure(content);

    Ok(GerberMetadata {
        cad_program,
        creation_date,
        layer_type,
        side,
        units,
        coordinate_format,
        statistics: analysis.statistics,
        apertures: analysis.apertures,
        d_code_validation: analysis.validation,
        structure_validation,
    })
}

fn extended_command<'a>(content: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("%{name}");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find("*%")?;
    Some(rest[..end].trim())
}

fn parse_coordinate_format(content: &str) -> Option<GerberCoordinateFormat> {
    let value = extended_command(content, "FS")?;
    let x_position = value.find('X')?;
    let y_position = value.find('Y')?;
    let parse_digits = |position: usize| {
        let digits = value.as_bytes().get(position + 1..position + 3)?;
        if !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
        Some((digits[0] - b'0', digits[1] - b'0'))
    };
    let (x_integer_digits, x_decimal_digits) = parse_digits(x_position)?;
    let (y_integer_digits, y_decimal_digits) = parse_digits(y_position)?;
    let flags = &value[..x_position];
    let notation = if flags.contains('A') {
        Some("absolute".to_owned())
    } else if flags.contains('I') {
        Some("incremental".to_owned())
    } else {
        None
    };
    let zero_suppression = if flags.contains('L') {
        Some("leading".to_owned())
    } else if flags.contains('T') {
        Some("trailing".to_owned())
    } else {
        None
    };

    Some(GerberCoordinateFormat {
        x_integer_digits,
        x_decimal_digits,
        y_integer_digits,
        y_decimal_digits,
        notation,
        zero_suppression,
    })
}

#[derive(Clone, Copy)]
enum Interpolation {
    Linear,
    Arc,
}

struct GerberGraphicsAnalysis {
    statistics: GerberStatistics,
    apertures: Vec<GerberAperture>,
    validation: GerberDCodeValidation,
}

fn analyze_graphics(content: &str) -> GerberGraphicsAnalysis {
    let mut statistics = GerberStatistics::default();
    let mut used_d_codes = BTreeSet::new();
    let mut apertures = BTreeMap::new();
    let mut duplicate_definitions = BTreeSet::new();
    let mut usage_counts: BTreeMap<u32, u64> = BTreeMap::new();
    let mut interpolation = Interpolation::Linear;
    let mut operation: Option<u32> = None;
    let mut current_aperture: Option<u32> = None;
    let bytes = content.as_bytes();
    let mut position = 0;

    while position < bytes.len() {
        if bytes[position].is_ascii_whitespace() {
            position += 1;
            continue;
        }
        if bytes[position] == b'%' {
            let block_start = position + 1;
            let Some(relative_end) = content[block_start..].find('%') else {
                break;
            };
            let block_end = block_start + relative_end;
            let block = content[block_start..block_end].trim_start();
            if block.starts_with("ADD") {
                statistics.aperture_count += 1;
                if let Some(aperture) = parse_aperture(block) {
                    if apertures.contains_key(&aperture.d_code) {
                        duplicate_definitions.insert(aperture.d_code);
                    } else {
                        apertures.insert(aperture.d_code, aperture);
                    }
                }
            } else if block.starts_with("AM") {
                statistics.aperture_macro_count += 1;
            }
            position = block_end + 1;
            continue;
        }

        let command_start = position;
        let Some(relative_end) = content[command_start..].find('*') else {
            break;
        };
        let command_end = command_start + relative_end;
        let command: String = content[command_start..command_end]
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        position = command_end + 1;

        if command.is_empty() || command.starts_with("G04") {
            continue;
        }
        if command.contains("G01") {
            interpolation = Interpolation::Linear;
        }
        if command.contains("G02") || command.contains("G03") {
            interpolation = Interpolation::Arc;
        }
        if command.contains("G36") {
            statistics.region_count += 1;
        }

        let command_d_codes = d_codes(&command);
        for code in &command_d_codes {
            if *code >= 10 {
                used_d_codes.insert(*code);
                current_aperture = Some(*code);
            }
        }
        if let Some(code) = command_d_codes.into_iter().rev().find(|code| *code <= 3) {
            operation = Some(code);
        }

        if !command.contains('X') && !command.contains('Y') {
            continue;
        }
        match operation {
            Some(1) => match interpolation {
                Interpolation::Linear => statistics.line_count += 1,
                Interpolation::Arc => statistics.arc_count += 1,
            },
            Some(3) => statistics.flash_count += 1,
            _ => {}
        }
        if matches!(operation, Some(1 | 3)) {
            if let Some(d_code) = current_aperture {
                *usage_counts.entry(d_code).or_default() += 1;
            }
        }
    }

    statistics.used_d_codes = used_d_codes.iter().copied().collect();
    let defined_d_codes: BTreeSet<_> = apertures.keys().copied().collect();
    let validation = GerberDCodeValidation {
        undefined_used: used_d_codes.difference(&defined_d_codes).copied().collect(),
        unused_defined: defined_d_codes.difference(&used_d_codes).copied().collect(),
        duplicate_definitions: duplicate_definitions.into_iter().collect(),
    };
    let apertures = apertures
        .into_values()
        .map(|mut aperture| {
            aperture.usage_count = usage_counts.get(&aperture.d_code).copied().unwrap_or(0);
            aperture
        })
        .collect();

    GerberGraphicsAnalysis {
        statistics,
        apertures,
        validation,
    }
}

fn parse_aperture(block: &str) -> Option<GerberAperture> {
    let value = block.trim().trim_end_matches('*').strip_prefix("ADD")?;
    let code_length = value.bytes().take_while(u8::is_ascii_digit).count();
    if code_length == 0 {
        return None;
    }
    let d_code = value[..code_length].parse().ok()?;
    if d_code < 10 {
        return None;
    }
    let definition = &value[code_length..];
    let (template, modifiers) = definition
        .split_once(',')
        .map(|(template, modifiers)| {
            (
                template,
                modifiers
                    .split('X')
                    .map(str::trim)
                    .filter(|modifier| !modifier.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or((definition, Vec::new()));
    if template.is_empty() {
        return None;
    }
    let (shape, has_hole) = match template {
        "C" => ("circle", Some(modifiers.len() > 1)),
        "R" => ("rectangle", Some(modifiers.len() > 2)),
        "O" => ("obround", Some(modifiers.len() > 2)),
        "P" => ("polygon", Some(modifiers.len() > 3)),
        _ => ("macro", None),
    };

    Some(GerberAperture {
        d_code,
        template: template.to_owned(),
        shape: shape.to_owned(),
        modifiers,
        has_hole,
        usage_count: 0,
    })
}

fn validate_structure(content: &str) -> GerberStructureValidation {
    let mut issues = Vec::new();
    let mut deprecated_commands = BTreeSet::new();
    let mut current_aperture = None;
    let mut operation = None;
    let mut region_depth = 0_u32;
    let mut saw_end = false;
    let mut reported_missing_aperture = false;
    let bytes = content.as_bytes();
    let mut position = 0;

    if extended_command(content, "FS").is_none() {
        push_issue(
            &mut issues,
            "missing_fs",
            "error",
            "Не найдена команда формата координат FS",
            None,
        );
    }
    if extended_command(content, "MO").is_none() {
        push_issue(
            &mut issues,
            "missing_mo",
            "error",
            "Не найдена команда единиц измерения MO",
            None,
        );
    }

    while position < bytes.len() {
        if bytes[position].is_ascii_whitespace() {
            position += 1;
            continue;
        }
        if bytes[position] == b'%' {
            let block_start = position + 1;
            let Some(relative_end) = content[block_start..].find('%') else {
                push_issue(
                    &mut issues,
                    "unterminated_extended_command",
                    "error",
                    "Расширенная команда не закрыта символом %",
                    Some(short_command(&content[position..])),
                );
                break;
            };
            let block_end = block_start + relative_end;
            let block = content[block_start..block_end].trim();
            let prefix: String = block.chars().take(2).collect();
            if block.starts_with("ADD") && !valid_aperture_definition(block) {
                push_issue(
                    &mut issues,
                    "malformed_aperture",
                    "error",
                    "Некорректное определение апертуры ADD",
                    Some(short_command(block)),
                );
            }
            if ["AS", "IN", "IP", "IR", "MI", "OF", "SF", "LN"].contains(&prefix.as_str()) {
                deprecated_commands.insert(prefix.clone());
            } else if ![
                "FS", "MO", "AD", "AM", "AB", "LP", "LM", "LR", "LS", "SR", "TF", "TA",
                "TO", "TD",
            ]
            .contains(&prefix.as_str())
            {
                push_issue(
                    &mut issues,
                    "unknown_extended_command",
                    "warning",
                    "Неизвестная расширенная команда",
                    Some(short_command(block)),
                );
            }
            position = block_end + 1;
            continue;
        }

        let command_start = position;
        let Some(relative_end) = content[command_start..].find('*') else {
            let remainder = content[command_start..].trim();
            if !remainder.is_empty() {
                push_issue(
                    &mut issues,
                    "unterminated_command",
                    "error",
                    "Команда не завершена символом *",
                    Some(short_command(remainder)),
                );
            }
            break;
        };
        let command_end = command_start + relative_end;
        let command: String = content[command_start..command_end]
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        position = command_end + 1;
        if command.is_empty() || command.starts_with("G04") {
            continue;
        }

        for code in letter_codes(&command, b'G') {
            if [54, 55, 70, 71, 74, 90, 91].contains(&code) {
                deprecated_commands.insert(format!("G{code:02}"));
            } else if ![1, 2, 3, 36, 37, 75].contains(&code) {
                push_issue(
                    &mut issues,
                    "unknown_command",
                    "warning",
                    "Обнаружена неизвестная G-команда",
                    Some(short_command(&command)),
                );
            }
        }
        for code in letter_codes(&command, b'M') {
            if code == 2 {
                saw_end = true;
            } else {
                push_issue(
                    &mut issues,
                    "unknown_command",
                    "warning",
                    "Обнаружена неизвестная M-команда",
                    Some(short_command(&command)),
                );
            }
        }
        if command.contains('N') {
            deprecated_commands.insert("N (sequence number)".into());
        }
        if command.bytes().any(|byte| {
            byte.is_ascii_alphabetic() && !matches!(byte, b'G' | b'D' | b'X' | b'Y' | b'I' | b'J' | b'M' | b'N')
        }) {
            push_issue(
                &mut issues,
                "unknown_command",
                "warning",
                "Команда содержит неизвестное поле",
                Some(short_command(&command)),
            );
        }

        if command.contains("G36") {
            if region_depth > 0 {
                push_issue(
                    &mut issues,
                    "nested_region",
                    "error",
                    "Область G36 начата до закрытия предыдущей области",
                    Some(short_command(&command)),
                );
            }
            region_depth += 1;
        }
        if command.contains("G37") {
            if region_depth == 0 {
                push_issue(
                    &mut issues,
                    "unmatched_region_end",
                    "error",
                    "Команда G37 не имеет соответствующей G36",
                    Some(short_command(&command)),
                );
            } else {
                region_depth -= 1;
            }
        }

        let command_d_codes = d_codes(&command);
        if let Some(code) = command_d_codes.iter().rev().find(|code| **code >= 10) {
            current_aperture = Some(*code);
        }
        if let Some(code) = command_d_codes.into_iter().rev().find(|code| *code <= 3) {
            operation = Some(code);
        }
        let has_coordinates = command.contains('X') || command.contains('Y');
        if has_coordinates
            && matches!(operation, Some(1 | 3))
            && current_aperture.is_none()
            && !reported_missing_aperture
        {
            reported_missing_aperture = true;
            push_issue(
                &mut issues,
                "coordinates_without_aperture",
                "error",
                "Операция рисования или вспышки выполняется без выбранной апертуры",
                Some(short_command(&command)),
            );
        }
    }

    if region_depth > 0 {
        push_issue(
            &mut issues,
            "unclosed_region",
            "error",
            "Область G36 не закрыта командой G37",
            None,
        );
    }
    if !saw_end {
        push_issue(
            &mut issues,
            "missing_eof",
            "error",
            "Не найдена завершающая команда M02",
            None,
        );
    }
    for command in deprecated_commands {
        push_issue(
            &mut issues,
            "deprecated_command",
            "warning",
            "Используется устаревшая команда Gerber",
            Some(command),
        );
    }

    GerberStructureValidation {
        is_valid: !issues.iter().any(|issue| issue.severity == "error"),
        issues,
    }
}

fn valid_aperture_definition(block: &str) -> bool {
    let Some(aperture) = parse_aperture(block) else {
        return false;
    };
    let required = match aperture.shape.as_str() {
        "circle" => 1,
        "rectangle" | "obround" | "polygon" => 2,
        _ => return true,
    };
    aperture.modifiers.len() >= required
        && aperture.modifiers.iter().all(|modifier| {
            modifier
                .parse::<f64>()
                .is_ok_and(|value| value.is_finite())
        })
}

fn letter_codes(command: &str, letter: u8) -> Vec<u32> {
    let bytes = command.as_bytes();
    let mut result = Vec::new();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] != letter {
            position += 1;
            continue;
        }
        let start = position + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            if let Ok(code) = command[start..end].parse() {
                result.push(code);
            }
        }
        position = end.max(position + 1);
    }
    result
}

fn short_command(command: &str) -> String {
    let mut value: String = command.chars().take(80).collect();
    if command.chars().count() > 80 {
        value.push('…');
    }
    value
}

fn push_issue(
    issues: &mut Vec<GerberStructureIssue>,
    code: &str,
    severity: &str,
    message: &str,
    command: Option<String>,
) {
    if issues
        .iter()
        .any(|issue| issue.code == code && issue.command == command)
    {
        return;
    }
    issues.push(GerberStructureIssue {
        code: code.into(),
        severity: severity.into(),
        message: message.into(),
        command,
    });
}

fn d_codes(command: &str) -> Vec<u32> {
    let bytes = command.as_bytes();
    let mut result = Vec::new();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] != b'D' {
            position += 1;
            continue;
        }
        let start = position + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            if let Ok(code) = command[start..end].parse() {
                result.push(code);
            }
        }
        position = end.max(position + 1);
    }
    result
}

fn attribute_fields<'a>(content: &'a str, name: &str) -> Option<Vec<String>> {
    let marker = format!("%TF.{name}");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find("*%")?;
    let value = rest[..end].trim().strip_prefix(',').unwrap_or("");

    Some(
        value
            .split(',')
            .map(|field| field.trim().to_owned())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_attributes_units_and_coordinate_format() {
        let source = "%FSLAX46Y46*%\n%MOMM*%\n%TF.FileFunction,Copper,L1,Top,Signal*%\n%TF.GenerationSoftware,KiCad,Pcbnew,8.0.5*%\n%TF.CreationDate,2026-10-14T08:32:10+03:00*%\nM02*";
        let metadata = parse_metadata(source).unwrap();

        assert_eq!(
            metadata.cad_program.as_deref(),
            Some("KiCad · Pcbnew · 8.0.5")
        );
        assert_eq!(
            metadata.creation_date.as_deref(),
            Some("2026-10-14T08:32:10+03:00")
        );
        assert_eq!(metadata.layer_type.as_deref(), Some("Copper"));
        assert_eq!(metadata.side.as_deref(), Some("top"));
        assert_eq!(metadata.units.as_deref(), Some("mm"));
        assert_eq!(
            metadata.coordinate_format,
            Some(GerberCoordinateFormat {
                x_integer_digits: 4,
                x_decimal_digits: 6,
                y_integer_digits: 4,
                y_decimal_digits: 6,
                notation: Some("absolute".into()),
                zero_suppression: Some("leading".into()),
            })
        );
    }

    #[test]
    fn accepts_gerber_without_optional_attributes() {
        let metadata = parse_metadata("%FSLAX24Y24*%\n%MOIN*%\nM02*").unwrap();
        assert_eq!(metadata.cad_program, None);
        assert_eq!(metadata.creation_date, None);
        assert_eq!(metadata.layer_type, None);
        assert_eq!(metadata.side, None);
        assert_eq!(metadata.units.as_deref(), Some("inch"));
        assert_eq!(metadata.coordinate_format.unwrap().x_decimal_digits, 4);
        assert_eq!(metadata.statistics, GerberStatistics::default());
    }

    #[test]
    fn counts_graphics_apertures_macros_and_used_d_codes() {
        let source = "%FSLAX46Y46*%\n%MOMM*%\n%ADD10C,0.200*%\n%ADD11R,0.500X0.300*%\n%AMRoundDonut*1,1,1.0,0,0*%\n%ADD12RoundDonut*%\nG04 D99 in a comment*\nD10*\nX0Y0D02*\nG01X100Y0D01*\nX200Y0*\nG02X300Y100I50J50D01*\nD11*\nX400Y200D03*\nG36*\nG01X0Y0D02*\nX0Y100D01*\nX100Y100D01*\nG37*\nM02*";
        let metadata = parse_metadata(source).unwrap();
        let statistics = metadata.statistics;

        assert_eq!(statistics.line_count, 4);
        assert_eq!(statistics.arc_count, 1);
        assert_eq!(statistics.flash_count, 1);
        assert_eq!(statistics.region_count, 1);
        assert_eq!(statistics.aperture_count, 3);
        assert_eq!(statistics.aperture_macro_count, 1);
        assert_eq!(statistics.used_d_codes, vec![10, 11]);
        assert_eq!(metadata.apertures.len(), 3);
        assert_eq!(metadata.apertures[0].shape, "circle");
        assert_eq!(metadata.apertures[0].modifiers, vec!["0.200"]);
        assert_eq!(metadata.apertures[0].has_hole, Some(false));
        assert_eq!(metadata.apertures[0].usage_count, 3);
        assert_eq!(metadata.apertures[1].shape, "rectangle");
        assert_eq!(metadata.apertures[1].usage_count, 3);
        assert_eq!(metadata.apertures[2].shape, "macro");
        assert_eq!(metadata.apertures[2].has_hole, None);
        assert_eq!(metadata.d_code_validation.undefined_used, Vec::<u32>::new());
        assert_eq!(metadata.d_code_validation.unused_defined, vec![12]);
    }

    #[test]
    fn reports_undefined_unused_and_duplicate_d_codes() {
        let source = "%FSLAX24Y24*%\n%MOMM*%\n%ADD10C,0.2*%\n%ADD10C,0.3*%\n%ADD12C,0.4*%\nD10*\nX0Y0D03*\nD11*\nX100Y100D03*\nM02*";
        let validation = parse_metadata(source).unwrap().d_code_validation;

        assert_eq!(validation.undefined_used, vec![11]);
        assert_eq!(validation.unused_defined, vec![12]);
        assert_eq!(validation.duplicate_definitions, vec![10]);
    }

    #[test]
    fn accepts_a_structurally_valid_file() {
        let source = "%FSLAX24Y24*%\n%MOMM*%\n%ADD10C,0.2*%\nD10*\nX0Y0D02*\nX100Y100D01*\nM02*";
        let validation = parse_metadata(source).unwrap().structure_validation;

        assert!(validation.is_valid);
        assert!(validation.issues.is_empty());
    }

    #[test]
    fn accepts_utf8_bom_without_false_structure_issues() {
        let source = "\u{feff}%FSLAX24Y24*%\n%MOMM*%\n%ADD10C,0.2*%\nD10*\nX0Y0D03*\nM02*";
        let validation = parse_metadata(source).unwrap().structure_validation;

        assert!(validation.is_valid);
        assert!(validation.issues.is_empty());
    }

    #[test]
    fn reports_structural_errors_and_deprecated_commands() {
        let source = "%FSLAX24Y24*%\n%ADD10C*%\nG74*\nG36*\nX0Y0D01*\nG99*";
        let validation = parse_metadata(source).unwrap().structure_validation;
        let issue_codes: BTreeSet<_> = validation
            .issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect();

        assert!(!validation.is_valid);
        assert!(issue_codes.contains("missing_mo"));
        assert!(issue_codes.contains("malformed_aperture"));
        assert!(issue_codes.contains("coordinates_without_aperture"));
        assert!(issue_codes.contains("unclosed_region"));
        assert!(issue_codes.contains("missing_eof"));
        assert!(issue_codes.contains("unknown_command"));
        assert!(validation.issues.iter().any(|issue| {
            issue.code == "deprecated_command"
                && issue.severity == "warning"
                && issue.command.as_deref() == Some("G74")
        }));
    }

    #[test]
    fn rejects_unrelated_text() {
        assert!(parse_metadata("hello world").is_err());
    }
}
