use std::num;
use thiserror::Error;

/// The errors may occur when querying with browserslist.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum Error {
    /// Failed to parse version string.
    #[error("invalid version string: {0:?}")]
    ParseVersion(num::ParseFloatError),

    /// Failed to parse number as percentage value.
    #[error("invalid percentage string: {0:?}")]
    ParsePercentage(num::ParseFloatError),

    /// Failed to parse number as versions count.
    #[error("invalid versions count: {0:?}")]
    ParseVersionsCount(num::ParseIntError),

    /// Failed to parse number as years count.
    #[error("invalid years count: {0:?}")]
    ParseYearsCount(num::ParseFloatError),

    /// Date format is invalid.
    #[error("invalid date: {0}")]
    InvalidDate(String),

    /// The given browser name can't be found.
    #[error("unknown browser: '{0}'")]
    BrowserNotFound(String),

    /// The given Electron version can't be found.
    #[error("unknown Electron version: {0}")]
    UnknownElectronVersion(String),

    /// The given Node.js version can't be found.
    #[error("unknown Node.js version: {0}")]
    UnknownNodejsVersion(String),

    /// The given version of the given browser can't be found.
    #[error("unknown version '{1}' of browser '{0}'")]
    UnknownBrowserVersion(String, String),

    /// Current environment doesn't support querying `current node`,
    /// for example, running this library on Non-Node.js platform or
    /// no Node.js installed.
    #[error("current environment for querying `current node` is not supported")]
    UnsupportedCurrentNode,

    /// Version string is missing when querying.
    #[error("specify versions in Browserslist query for browser '{0}'")]
    VersionRequired(String),

    /// Query can't be recognized.
    #[error("unknown browser query: '{0}'")]
    UnknownQuery(String),

    /// Duplicated section in configuration.
    #[error("duplicated section '{0}' in config")]
    DuplicatedSection(String),

    /// Failed to read config.
    #[error("failed to read config file: {0}")]
    FailedToReadConfig(String),

    /// Missing `browserslist` field in `package.json` file.
    #[error("missing 'browserslist' field in '{0}' file")]
    MissingFieldInPkg(String),

    /// Duplicated configuration found.
    #[error("duplicated: '{0}' directory contains both {1} and {2}.")]
    DuplicatedConfig(String, &'static str, &'static str),
}
