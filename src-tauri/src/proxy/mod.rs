pub mod client;
pub mod command;
pub mod schema;

mod mappers;
pub(crate) mod utils;

pub(crate) use schema::proxy_scheme_handler;
pub use tauri::http;
pub use tauri::http::request::Builder as RequestBuilder;
pub use tauri::http::response::Builder as ResponseBuilder;
pub(crate) use utils::PROTOCOL_URL;

// We use tauri-exported http as the base, as it is the platform we're building on.
// TODO: NewType these with Deref? We could implement mappers?
pub(crate) type Request = tauri::http::Request<Vec<u8>>;
pub(crate) type Response = tauri::http::Response<Vec<u8>>;
pub(crate) type HttpResult = Result<Response, crate::errors::Error>;
