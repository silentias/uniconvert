// Заглушки намеренно ещё не используют аргументы и внутренние типы.
#![allow(dead_code, unused_variables)]

pub mod analysis;
pub mod audio;
pub mod converters;
pub mod domain;
pub mod infra;
pub mod jobs;
pub mod pipeline;
pub mod presets;

pub struct CoreServices;

impl CoreServices {
    pub fn new() -> Self {
        // TODO: Сначала собрать пустые реестры конвертеров и пресетов.
        // TODO: Затем подключить очередь, анализаторы и системные возможности.
        // TODO: Позже разрешить внедрение адаптеров для CLI, Tauri и тестов.
        todo!("core composition is not implemented")
    }
}
