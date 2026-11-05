use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};

pub fn execute(request: &ConvertRequest, args: &[String]) -> Result<ConvertOutput, ConvertError> {
    // TODO: Сначала запустить FFmpeg с готовыми аргументами.
    // TODO: Затем разобрать прогресс и обеспечить отмену.
    // TODO: Позже добавить восстановление и подробные логи.
    todo!("FFmpeg execution is not implemented")
}
