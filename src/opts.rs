use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
/// Options for controlling the behavior of browserslist.
pub struct Opts {
    /// Use desktop browsers if Can I Use doesn’t have data about this mobile version.
    pub mobile_to_desktop: bool,

    /// If `true`, ignore unknown versions then return empty result;
    /// otherwise, reject with an error.
    pub ignore_unknown_versions: bool,

    /// Path to configuration file with queries.
    pub config: Option<String>,

    /// Processing environment. It will be used to take right queries from config file.
    pub env: Option<String>,

    /// File or directory path for looking for configuration file.
    pub path: Option<String>,

    /// Throw error on missing env.
    pub throw_on_missing: bool,

    /// Disable security checks for `extends` query.
    pub dangerous_extend: bool,
}
