// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // TODO: Сначала запускать Tauri-обёртку приложения.
    // TODO: Позже установить раннюю диагностику panic и платформенные хуки.
    uniconvert_lib::run()
}
