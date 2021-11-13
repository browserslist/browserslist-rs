use serde::{Deserialize, Serialize};

/// Options for controlling the behavior of browserslist.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Opts {
    /// Use desktop browsers if Can I Use doesn’t have data about this mobile version.
    pub(crate) mobile_to_desktop: bool,

    /// If `true`, ignore unknown versions then return empty result;
    /// otherwise, throw an error.
    pub(crate) ignore_unknown_versions: bool,
}

impl Opts {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get `mobile_to_desktop` option.
    #[inline]
    pub fn is_mobile_to_desktop(&self) -> bool {
        self.mobile_to_desktop
    }

    /// Set `mobile_to_desktop` option.
    pub fn mobile_to_desktop(&mut self, flag: bool) -> &mut Self {
        self.mobile_to_desktop = flag;
        self
    }

    /// Get `ignore_unknown_versions` options.
    #[inline]
    pub fn is_ignore_unknown_versions(&self) -> bool {
        self.ignore_unknown_versions
    }

    /// Set `ignore_unknown_versions` option.
    pub fn ignore_unknown_versions(&mut self, flag: bool) -> &mut Self {
        self.ignore_unknown_versions = flag;
        self
    }
}
