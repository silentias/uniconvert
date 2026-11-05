use crate::domain::ConvertError;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::{bin_resolver, process};

pub fn probe_json(path: &Path) -> Result<String, ConvertError> {
    probe_json_cancellable(path, &AtomicBool::new(false))
}

pub fn probe_json_cancellable(path: &Path, cancelled: &AtomicBool) -> Result<String, ConvertError> {
    // TODO: Затем валидировать версии ffprobe и необычные схемы потоков.
    let ffprobe = bin_resolver::resolve("ffprobe")?;
    let args = vec![
        "-v".into(),
        "error".into(),
        "-show_format".into(),
        "-show_streams".into(),
        "-of".into(),
        "json".into(),
        path.to_string_lossy().into_owned(),
    ];
    let output = process::run_cancellable(&ffprobe.to_string_lossy(), &args, cancelled)?;
    if !output.status.success() {
        return Err(ConvertError::ProcessFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| ConvertError::ProcessFailed(error.to_string()))
}
