#[cfg(feature = "node")]
use napi_derive::*;
use serde::{Deserialize, Serialize};

/// Options for controlling the behavior of browserslist.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Opts<'a> {
    #[serde(default)]
    pub(crate) mobile_to_desktop: bool,

    #[serde(default)]
    pub(crate) ignore_unknown_versions: bool,

    #[serde(default)]
    pub(crate) config: Option<&'a str>,

    #[serde(default)]
    pub(crate) env: Option<&'a str>,

    #[serde(default)]
    pub(crate) path: Option<&'a str>,
}

impl<'a> Opts<'a> {
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
    pub fn config(&mut self, config_path: &'a str) -> &mut Self {
        self.config = Some(config_path);
        self
    }

    /// Processing environment. It will be used to take right queries from config file.
    pub fn env(&mut self, env: &'a str) -> &mut Self {
        self.env = Some(env);
        self
    }

    /// File or directory path for looking for configuration file.
    pub fn path(&mut self, path: &'a str) -> &mut Self {
        self.path = Some(path);
        self
    }
}
