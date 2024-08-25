use crate::{
    app::MAIN_WINDOW_LABEL,
    errors::Error,
    proxy::{http::header::LOCATION, Response},
};
use tauri::{AppHandle, Manager};

pub(crate) const PROTOCOL_URL: &str = if cfg!(target_os = "android") {
    "http://proxy.localhost"
} else {
    "proxy://localhost"
};

/// Handle HTTP redirect to given location.
/// This is triggered by "interceptors" when the remote server wants to redirect the client to another url.
/// Due to certain limitations (such as lack of support on custom protocols),
///  we can't rely on webviews to handle this properly.
// TODO: Should this be part of another "upper" object along with ProxyClient, like "Proxy".
pub(super) fn handle_redirect(app: &AppHandle, response: &Response) -> Result<(), Error> {
    if response.status().is_redirection() {
        match response.headers().get(LOCATION) {
            Some(location) => {
                let location = location.to_str().map_err(|e| {
                    tracing::error!("Failed to parse Location header value to str: {e}");
                    Error::ParseError("Failed to parse location header for redirect".to_string())
                })?;

                // TODO: Sanitize and truncate location to prevent js-injection through this.
                let window = app
                    .get_webview_window(MAIN_WINDOW_LABEL)
                    .ok_or(Error::ProxyError(
                        "Failed to find window to handle redirect.".to_string(),
                    ))?;

                window
                    .eval(format!("window.location = '{PROTOCOL_URL}/{location}'").as_str())
                    .map_err(|e| {
                        tracing::error!("Executing JS to handle proxy redirect failed: {e}");
                        Error::ProxyError("Failed to handle redirect on the window.".to_string())
                    })?;
                Ok(())
            }
            None => Err(Error::ProxyError(
                "Failed to handle redirect: Missing Location header".to_string(),
            )),
        }?
    }

    Ok(())
}
