use std::num;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid version string: {0:?}")]
    ParseVersion(num::ParseFloatError),
    #[error("invalid percentage string: {0:?}")]
    ParsePercentage(num::ParseFloatError),
    #[error("invalid versions count: {0:?}")]
    ParseVersionsCount(num::ParseIntError),
    #[error("unknown browser: '{0}'")]
    BrowserNotFound(String),
}
