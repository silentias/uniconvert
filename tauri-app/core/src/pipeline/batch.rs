use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};

pub fn execute_batch(requests: &[ConvertRequest]) -> Result<Vec<ConvertOutput>, ConvertError> {
    // TODO: Сначала обрабатывать файлы последовательно.
    // TODO: Затем добавить ограниченный параллелизм и общую отмену.
    // TODO: Позже поддержать политики частичных ошибок.
    todo!("batch pipeline is not implemented")
}
