use crate::domain::ConvertError;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn resolve(binary: &str) -> Result<PathBuf, ConvertError> {
    // TODO: Сначала найти bundled sidecar для текущей платформы.
    // TODO: Затем поддержать пользовательский и системный FFmpeg.
    // TODO: Позже проверять версию и целостность бинарника.
    let environment_variable = format!("UNICONVERT_{}_PATH", binary.to_ascii_uppercase());
    if let Some(path) = std::env::var_os(&environment_variable) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    let executable_name = if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.into()
    };
    let manifest_binary = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../binaries")
        .join(&executable_name);
    if manifest_binary.is_file() {
        return Ok(manifest_binary);
    }

    if let Ok(current_exe) = std::env::current_exe() {
        for candidate in [
            current_exe.parent().map(|path| path.join(&executable_name)),
            current_exe
                .parent()
                .map(|path| path.join("binaries").join(&executable_name)),
        ]
        .into_iter()
        .flatten()
        {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if Command::new(&executable_name)
        .arg("-version")
        .output()
        .is_ok()
    {
        return Ok(PathBuf::from(executable_name));
    }

    Err(ConvertError::ProcessFailed(format!(
        "{binary} не найден. Добавьте его в tauri-app/binaries или задайте {environment_variable}"
    )))
}
