use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to reach qBittorrent: `{0}`")]
    BadGateway(reqwest::Error),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Unable to parse qBittorrent data: `{0}`")]
    InvalidResponse(reqwest::Error),
    
    #[error("Invalid tag: `{0}`")]
    InvalidTag(String),
    
    #[error("Invalid regex pattern: `{0}`")]
    InvalidRegex(String),
    
    #[error("Offset must be an integer, found: `{0}`")]
    InvalidOffset(String),
    
    #[error("`{0}` contains no matches with pattern `{1}`")]
    NoMatch(String, String),
    
    #[error("Pattern `{0}` does not contain a group")]
    NoGroup(String),
    
    #[error("Group `{0}` is not a number")]
    GroupNotANumber(String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::BadGateway(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message: String = match &self {
            Self::BadGateway(error) => format!("{}: {}", self, error),
            _ => self.to_string(),
        };
        
        error!(message);

        (self.status_code(), message).into_response()
    }
}
