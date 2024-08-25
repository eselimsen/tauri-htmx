use crate::{
    errors::Error,
    proxy::{client::ProxyClient, utils::handle_redirect, Request},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, State};

#[derive(Serialize, Deserialize)]
pub struct CmdProxyRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Serialize)]
pub struct CmdProxyResponse {
    // Okay for now, as we return just string responses from the htmx calls,
    //  but would be problematic with binary responses.
    pub response: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
}

#[tauri::command]
pub async fn proxy_request(
    request: CmdProxyRequest,
    proxy_client: State<'_, ProxyClient>,
    app: AppHandle,
) -> Result<CmdProxyResponse, Error> {
    tracing::debug!("Proxying request from Tauri command.");
    let request = Request::try_from(request)?;
    let proxy_response = proxy_client.handle_proxy_request(request).await?;

    handle_redirect(&app, &proxy_response)?;

    let response = CmdProxyResponse::from(proxy_response);
    Ok(response)
}
