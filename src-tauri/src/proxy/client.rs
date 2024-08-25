use crate::{
    errors::Error,
    proxy::{http::header, HttpResult, Request, ResponseBuilder},
};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Client,
};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
use std::sync::Arc;

const ALLOWED_REQUEST_HEADERS: [HeaderName; 5] = [
    header::ACCEPT,
    header::ACCEPT_CHARSET,
    header::ACCEPT_ENCODING,
    header::ACCEPT_LANGUAGE,
    header::CONTENT_TYPE,
];

const ALLOWED_RESPONSE_HEADERS: [HeaderName; 4] = [
    header::CONTENT_TYPE,
    header::DATE,
    header::LOCATION,
    header::VARY,
];

const COOKIES_FILE_PATH: &str = "proxy_cookies.json";

const LOGIN_PATH: &str = "app/auth";

const LOGOUT_PATH: &str = "app/logout";

const TOKEN_PATHS: [&str; 2] = [LOGIN_PATH, LOGOUT_PATH];

const USER_AGENT: &str = concat!("TauriHtmxApp", "/", env!("CARGO_PKG_VERSION"));

pub(crate) struct ProxyClient {
    /// Base url of the remote server, without the trailing slash.
    base_url: String,
    client: Client,
    cookie_store: Arc<CookieStoreRwLock>,
}

impl ProxyClient {
    pub fn new(base_url: String) -> Result<Self, Error> {
        let cookie_store = Arc::new(Self::load_cookie_store());
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_provider(Arc::clone(&cookie_store))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            base_url,
            client,
            cookie_store: Arc::clone(&cookie_store),
        })
    }

    fn is_header_allowed(header_name: &HeaderName, whitelist: &[HeaderName]) -> bool {
        // Allow whitelisted, or Htmx headers.
        whitelist.contains(header_name) || header_name.as_str().to_lowercase().starts_with("hx-")
    }

    fn filter_headers(source: &HeaderMap, whitelist: &[HeaderName]) -> HeaderMap {
        source
            .iter()
            .filter_map(|(header_name, value)| {
                if Self::is_header_allowed(header_name, whitelist) {
                    Some((header_name.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn handle_proxy_request(&self, request: Request) -> HttpResult {
        let (parts, body) = request.into_parts();
        // Prepare remote request url and headers.
        let path = parts
            .uri
            .path_and_query()
            .map(|p| p.to_string())
            .unwrap_or_default();
        let path = path.trim_start_matches("/");
        let url = format!("{}/{}", self.base_url, path);

        tracing::debug!("Proxy Path: {:?}", path);
        tracing::debug!("Proxy Url: {:?}", url);
        tracing::debug!("Proxy Body Size: {:?}", body.len());

        // Send the request to remote server.
        let request = self
            .client
            .request(parts.method, &url)
            .headers(Self::filter_headers(
                &parts.headers,
                &ALLOWED_REQUEST_HEADERS,
            ))
            .body(body);
        let remote_response = request.send().await?;

        // Persist cookies when token paths (login, logout, etc.) were hit.
        // Login will return success, logout will return redirection,
        //  we're safe to cover both, saving cookies is an idempotent action anyway.
        let remote_status = remote_response.status();
        if TOKEN_PATHS.contains(&path)
            && (remote_status.is_success() || remote_status.is_redirection())
        {
            self.save_cookie_store();
        }

        // Build the response from the remote response.
        let mut response = ResponseBuilder::new();
        response = response.status(remote_response.status().as_str());
        let response_headers =
            Self::filter_headers(remote_response.headers(), &ALLOWED_RESPONSE_HEADERS);

        for (header_name, value) in response_headers.iter() {
            response = response.header(header_name, value);
        }
        if !response_headers.contains_key(header::CONTENT_TYPE) {
            response = response.header(header::CONTENT_TYPE, "text/html");
        }

        Ok(response
            .body(remote_response.bytes().await?.to_vec())
            .map_err(|e| Error::ProxyError(format!("Failed to read remote response body: {e}")))?)
    }

    fn load_cookie_store() -> CookieStoreRwLock {
        let cookie_store = std::fs::File::open(COOKIES_FILE_PATH)
            .map(std::io::BufReader::new)
            .ok()
            .and_then(|reader| CookieStore::load_json(reader).ok());
        CookieStoreRwLock::new(cookie_store.unwrap_or_default())
    }

    fn save_cookie_store(&self) {
        // TODO: Save to db, encrypt etc.
        std::fs::File::create(COOKIES_FILE_PATH)
            .map(std::io::BufWriter::new)
            .map_err(|e| {
                tracing::error!(
                    "Failed to create proxy cookie store file ('{COOKIES_FILE_PATH}'): {e}"
                )
            })
            .and_then(|mut writer| {
                self.cookie_store
                    .write()
                    .map_err(|e| tracing::error!("Cookie store RwLock poisoned for write! {e}"))
                    .and_then(|cookie_store| {
                        cookie_store.save_json(&mut writer).map_err(|e| {
                            tracing::error!("Failed to store the cookies as json: {e}")
                        })
                    })
            })
            .ok();
    }
}
