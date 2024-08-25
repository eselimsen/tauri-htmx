use crate::{
    errors::Error,
    proxy::{
        client::ProxyClient,
        http::{header::CONTENT_TYPE, StatusCode},
        utils::handle_redirect,
        Request, Response, ResponseBuilder,
    },
};
use tauri::{AppHandle, Manager, State};

fn proxy_scheme_handler_inner(app: &AppHandle, request: Request) -> Result<Response, Error> {
    tauri::async_runtime::block_on(async {
        let proxy_client: State<'_, ProxyClient> = app.state();
        let proxy_response = proxy_client.handle_proxy_request(request).await?;

        handle_redirect(&app, &proxy_response)?;
        Ok(proxy_response)
    })
}

// Meant to serve assets primarily, http requests are intercepted for htmx.
// TODO: Implement async one instead...
pub(crate) fn proxy_scheme_handler(app: &AppHandle, request: Request) -> Response {
    tracing::debug!("Proxying request from custom scheme.");
    proxy_scheme_handler_inner(&app, request).unwrap_or_else(|error| {
        ResponseBuilder::new()
            .header(CONTENT_TYPE, "text/plain")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(error.to_string().as_bytes().to_vec())
            .map_err(|e| Error::InternalError(format!("Failed to build error response: {e}")))
            .unwrap_or_default()
    })
}
