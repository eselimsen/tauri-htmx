// TODO: Detail these errors, also use anyhow?
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // TODO: Use the inner error, extract more info from reqwest.
    #[error("Failed to build/send request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Error when handling proxy request: {0}")]
    ProxyError(String),
    #[error("Failed to parse value: {0}")]
    ParseError(String),
    #[error("Requested feature isn't implemented: {0}")]
    Unimplemented(String),
}

// TODO: Render to a common format, at least for Response types?
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
