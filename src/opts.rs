#[cfg(feature = "node")]
use napi_derive::*;
use serde::{Deserialize, Serialize};

/// Options for controlling the behavior of browserslist.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Opts {
    #[serde(default)]
    pub(crate) mobile_to_desktop: bool,

    #[serde(default)]
    pub(crate) ignore_unknown_versions: bool,

    #[serde(default)]
    pub(crate) config: Option<String>,

    #[serde(default)]
    pub(crate) env: Option<String>,

    #[serde(default)]
    pub(crate) path: Option<String>,
}

impl Opts {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use desktop browsers if Can I Use doesn’t have data about this mobile version.
    pub fn mobile_to_desktop(&mut self, flag: bool) -> &mut Self {
        self.mobile_to_desktop = flag;
        self
    }

    /// If `true`, ignore unknown versions then return empty result;
    /// otherwise, reject with an error.
    pub fn ignore_unknown_versions(&mut self, flag: bool) -> &mut Self {
        self.ignore_unknown_versions = flag;
        self
    }

    /// Path to configuration file with queries.
    pub fn config<S: AsRef<str>>(&mut self, config_path: S) -> &mut Self {
        self.config = Some(config_path.as_ref().to_string());
        self
    }

    /// Processing environment. It will be used to take right queries from config file.
    pub fn env<S: AsRef<str>>(&mut self, env: S) -> &mut Self {
        self.env = Some(env.as_ref().to_string());
        self
    }

    /// File or directory path for looking for configuration file.
    pub fn path<S: AsRef<str>>(&mut self, path: S) -> &mut Self {
        self.path = Some(path.as_ref().to_string());
        self
    }
}
