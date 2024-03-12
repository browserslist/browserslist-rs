use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
/// The errors may occur when querying with browserslist.
pub enum Error {
    #[error("failed to parse the rest of input: ...'{0}'")]
    /// Error of parsing query.
    Nom(String),

    #[error("invalid date: {0}")]
    /// Date format is invalid.
    InvalidDate(String),

    #[error("query cannot start with 'not'; add any other queries before '{0}'")]
    /// Query can't start with a negated query which starts with `not`.
    NotAtFirst(String),

    #[error("unknown browser: '{0}'")]
    /// The given browser name can't be found.
    BrowserNotFound(String),

    #[error("unknown Electron version: {0}")]
    /// The given Electron version can't be found.
    UnknownElectronVersion(String),

    #[error("unknown Node.js version: {0}")]
    /// The given Node.js version can't be found.
    UnknownNodejsVersion(String),

    #[error("unknown version '{1}' of browser '{0}'")]
    /// The given version of the given browser can't be found.
    UnknownBrowserVersion(String, String),

    #[error("current environment for querying `current node` is not supported")]
    /// Current environment doesn't support querying `current node`,
    /// for example, running this library on Non-Node.js platform or
    /// no Node.js installed.
    UnsupportedCurrentNode,

    #[error("current environment for querying `extends ...` is not supported")]
    /// Current environment doesn't support querying `extends`.
    UnsupportedExtends,

    #[error("unknown browser feature: '{0}'")]
    /// Unknown browser feature.
    UnknownBrowserFeature(String),

    #[error("unknown region: '{0}'")]
    /// Unknown Can I Use region.
    UnknownRegion(String),

    #[error("unknown browser query: '{0}'")]
    /// Query can't be recognized.
    UnknownQuery(String),

    #[error("duplicated section '{0}' in config")]
    /// Duplicated section in configuration.
    DuplicatedSection(String),

    #[error("failed to read config file: {0}")]
    /// Failed to read config.
    FailedToReadConfig(String),

    #[error("missing 'browserslist' field in '{0}' file")]
    /// Missing `browserslist` field in `package.json` file.
    MissingFieldInPkg(String),

    #[error("duplicated: '{0}' directory contains both {1} and {2}.")]
    /// Duplicated configuration found.
    DuplicatedConfig(String, &'static str, &'static str),

    #[error("failed to access current working directory")]
    /// Failed to access the current working directory.
    FailedToAccessCurrentDir,

    #[error("missing config for Browserslist environment '{0}'")]
    /// Missing config corresponding to specific environment.
    MissingEnv(String),

    #[error("invalid extend name: {0}")]
    /// Invalid extend name
    InvalidExtendName(&'static str),

    #[error("failed to resolve '{0}' package in `extends` query")]
    /// Failed to resolve package in `extends` query.
    FailedToResolveExtend(String),

    #[error("year overflow")]
    /// Year overflow.
    YearOverflow,
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(e: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Self::Nom(e.input.to_owned()),
            _ => unreachable!(),
        }
    }
}
