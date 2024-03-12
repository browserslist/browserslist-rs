use serde::{Deserialize, Serialize};

/// Options for controlling the behavior of browserslist.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Opts {
    pub mobile_to_desktop: bool,
    pub ignore_unknown_versions: bool,
    pub config: Option<String>,
    pub env: Option<String>,
    pub path: Option<String>,
    pub throw_on_missing: bool,

    /// Disable security checks for `extends` query.
    pub dangerous_extend: bool,
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

    /// Throw error on missing env.
    pub fn throw_on_missing(&mut self, flag: bool) -> &mut Self {
        self.throw_on_missing = flag;
        self
    }
}
