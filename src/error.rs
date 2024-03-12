use thiserror::Error;

/// The errors may occur when querying with browserslist.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum Error {
    /// Error of parsing query.
    #[error("failed to parse the rest of input: ...'{0}'")]
    Nom(String),

    /// Date format is invalid.
    #[error("invalid date: {0}")]
    InvalidDate(String),

    /// Query can't start with a negated query which starts with `not`.
    #[error("query cannot start with 'not'; add any other queries before '{0}'")]
    NotAtFirst(String),

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

    #[error("current environment for querying `extends ...` is not supported")]
    /// Current environment doesn't support querying `extends`.
    UnsupportedExtends,

    /// Unknown browser feature.
    #[error("unknown browser feature: '{0}'")]
    UnknownBrowserFeature(String),

    /// Unknown Can I Use region.
    #[error("unknown region: '{0}'")]
    UnknownRegion(String),

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

    /// Failed to access the current working directory.
    #[error("failed to access current working directory")]
    FailedToAccessCurrentDir,

    /// Missing config corresponding to specific environment.
    #[error("missing config for Browserslist environment '{0}'")]
    MissingEnv(String),

    #[error("invalid extend name: {0}")]
    /// invalid extend name
    InvalidExtendName(&'static str),

    #[error("failed to resolve '{0}' package in `extends` query")]
    /// Failed to resolve package in `extends` query.
    FailedToResolveExtend(String),
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(e: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Self::Nom(e.input.to_owned()),
            _ => unreachable!(),
        }
    }
}
