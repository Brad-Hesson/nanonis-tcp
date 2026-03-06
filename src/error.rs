#[derive(Debug, thiserror::Error)]
pub enum NanonisTcpError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("api error: {0}")]
    Api(String),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
}

pub type NanonisTcpResult<T> = std::result::Result<T, NanonisTcpError>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("wrong command name received: expected `{expected}` but got `{received}`")]
    NameMismatch { expected: String, received: String },
    #[error("did not parse all received bytes: expected `{expected}` but parsed `{parsed}`")]
    ReadLenMismatch { expected: usize, parsed: usize },
    #[error("did not write full buffer: expected `{expected}` but wrote `{wrote}`")]
    WriteLenMismatch { expected: usize, wrote: usize },
}
