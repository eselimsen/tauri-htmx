#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod errors;
mod proxy;

// TODO: Can we remove this?
fn main() {
    app::start_tauri_app();
}
