/// Options for controlling the behavior of browserslist.
#[derive(Clone, Debug, Default)]
pub struct Opts {
    /// Use desktop browsers if Can I Use doesn’t have data about this mobile version.
    pub mobile_to_desktop: bool,
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
}
