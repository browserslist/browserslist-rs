/// Options for controlling the behavior of browserslist.
#[derive(Clone, Debug, Default)]
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

    /// Set `mobile_to_desktop` option.
    pub fn mobile_to_desktop(self, flag: bool) -> Self {
        Self {
            mobile_to_desktop: flag,
            ..self
        }
    }

    /// Set `ignore_unknown_versions` option.
    pub fn ignore_unknown_versions(self, flag: bool) -> Self {
        Self {
            ignore_unknown_versions: flag,
            ..self
        }
    }
}
