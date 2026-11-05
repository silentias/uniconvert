// Фасады и события объявлены заранее и будут подключаться по мере реализации ядра.
#![allow(dead_code, unused_variables)]

mod commands;
mod events;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: Сначала зарегистрировать тонкие команды и обязательные плагины.
    // TODO: Затем создать AppState и связать события ядра с Tauri transport.
    // TODO: Позже добавить graceful shutdown, восстановление очереди и диагностику.
    tauri::Builder::default()
        .manage(state::ActiveJobs::default())
        .manage(commands::audio_preview::AudioPreviews::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::probe::probe_file,
            commands::audio_preview::prepare_audio_preview,
            commands::audio_preview::release_audio_preview,
            commands::convert::convert,
            commands::convert_audio::convert_audio,
            commands::convert_image::convert_image,
            commands::convert_video::convert_video,
            commands::cancel::cancel_convert,
            commands::estimate::estimate_output,
            commands::presets::list_presets,
            commands::formats::list_supported_formats,
            commands::gerber_metadata::extract_gerber_metadata,
            commands::image_metadata::extract_image_metadata,
            commands::queue::get_queue,
            commands::queue::clear_queue,
            commands::queue::reorder_queue,
            commands::capabilities::detect_capabilities,
            commands::preview::generate_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
