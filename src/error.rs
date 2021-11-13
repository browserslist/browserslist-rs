use std::num;
use thiserror::Error;

/// The errors may occur when querying with browserslist.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("invalid version string: {0:?}")]
    ParseVersion(num::ParseFloatError),
    #[error("invalid percentage string: {0:?}")]
    ParsePercentage(num::ParseFloatError),
    #[error("invalid versions count: {0:?}")]
    ParseVersionsCount(num::ParseIntError),
    #[error("invalid years count: {0:?}")]
    ParseYearsCount(num::ParseFloatError),
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("unknown browser: '{0}'")]
    BrowserNotFound(String),
    #[error("unknown Electron version: {0}")]
    UnknownElectronVersion(String),
    #[error("unknown Node.js version: {0}")]
    UnknownNodejsVersion(String),
    #[error("unknown version '{1}' of browser '{0}'")]
    UnknownBrowserVersion(String, String),
    #[error("current environment for querying `current node` is not supported")]
    UnsupportedCurrentNode,
    #[error("specify versions in Browserslist query for browser '{0}'")]
    VersionRequired(String),
    #[error("unknown browser query: '{0}'")]
    UnknownQuery(String),
}
