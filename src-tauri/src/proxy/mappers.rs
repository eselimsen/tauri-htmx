use crate::{
    errors::Error,
    proxy::{
        command::{CmdProxyRequest, CmdProxyResponse},
        http::{header::CONTENT_TYPE, Method, StatusCode},
        HttpResult, Request, RequestBuilder, Response, ResponseBuilder, PROTOCOL_URL,
    },
};
use std::str::FromStr;

impl TryFrom<CmdProxyRequest> for Request {
    type Error = Error;

    fn try_from(value: CmdProxyRequest) -> Result<Self, Self::Error> {
        let mut request = RequestBuilder::new()
            .uri(format!("{PROTOCOL_URL}/{}", value.path))
            .method(Method::from_str(&value.method).map_err(|_e| {
                Error::ParseError(format!("Invalid http method for request: {}", value.method))
            })?);

        for (header, value) in value.headers.iter() {
            request = request.header(header, value);
        }

        let request = request
            .body(value.body.unwrap_or_default().as_bytes().to_vec())
            .map_err(|e| {
                tracing::error!("Failed to parse ipc proxy request: {e}");
                Error::ProxyError("Failed to parse ipc proxy request.".to_string())
            })?;
        Ok(request)
    }
}

impl From<HttpResult> for CmdProxyResponse {
    fn from(value: HttpResult) -> Self {
        match value {
            Ok(value) => value.into(),
            Err(error) => ResponseBuilder::new()
                .header(CONTENT_TYPE, "text/plain")
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string().as_bytes().to_vec())
                .map_err(|e| Error::InternalError(format!("Failed to build error response: {e}")))
                .into(),
        }
    }
}

impl From<Response> for CmdProxyResponse {
    fn from(value: Response) -> Self {
        Self {
            response: String::from_utf8_lossy(value.body()).to_string(),
            status: value.status().as_u16(),
            headers: value
                .headers()
                .iter()
                .map(|(header, value)| {
                    (
                        header.to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect(),
        }
    }
}
