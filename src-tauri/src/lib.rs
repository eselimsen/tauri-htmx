mod app;
mod errors;
mod proxy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    app::start_tauri_app();
}
