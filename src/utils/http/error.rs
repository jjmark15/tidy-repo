use http_types::StatusCode;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    message: String,
    status: StatusCode,
}

impl Error {
    pub fn status_code(&self) -> &StatusCode {
        &self.status
    }
}

impl From<http_types::Error> for Error {
    fn from(from: http_types::Error) -> Self {
        Error {
            message: from.to_string(),
            status: from.status(),
        }
    }
}
