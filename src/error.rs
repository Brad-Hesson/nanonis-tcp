#[derive(Debug, thiserror::Error)]
pub enum NanonisError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("api error: {0}")]
    Api(String),
}

pub type NanonisResult<T> = std::result::Result<T, NanonisError>;
