use crate::proxy::{client::ProxyClient, proxy_scheme_handler};
use tauri::{Builder, Manager};
use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{layer::SubscriberExt, Registry};

// Default name assigned by Tauri, also set in tauri.conf.json.
pub const MAIN_WINDOW_LABEL: &str = "main";

fn init_logging() {
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(LevelFilter::DEBUG);
    set_global_default(subscriber).expect("Failed to set log subscriber");
}

pub fn start_tauri_app() {
    init_logging();

    let base_url = option_env!("TAURI_PROXY_BASE_URL").unwrap_or_else(|| "http://localhost:8003");
    let proxy_client =
        ProxyClient::new(base_url.to_string()).expect("Failed to build proxy client");

    Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            app.manage(proxy_client);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::proxy::command::proxy_request
        ])
        .register_uri_scheme_protocol("proxy", proxy_scheme_handler)
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
