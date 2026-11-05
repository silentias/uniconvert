use std::path::PathBuf;

use uniconvert_core::{
    analysis::gerber::{extract_metadata, GerberMetadata},
    domain::ConvertError,
};

#[tauri::command]
pub fn extract_gerber_metadata(path: PathBuf) -> Result<GerberMetadata, ConvertError> {
    extract_metadata(&path)
}
