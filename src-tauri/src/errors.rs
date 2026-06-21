use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("provider '{0}' not found")]
    ProviderNotFound(String),

    #[error("missing API key for provider '{0}' — set env var {1} or paste in Settings")]
    MissingKey(String, String),

    #[error("network error: {0}")]
    Network(String),

    #[error("upstream API error ({status}): {body}")]
    Upstream { status: u16, body: String },

    #[error("invalid response: {0}")]
    Parse(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("config error: {0}")]
    Config(String),
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Parse(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

// Tauri requires command errors to be Serialize
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
