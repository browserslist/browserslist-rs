#[derive(Clone, Debug, Default)]
pub struct Opts {
    pub mobile_to_desktop: bool,
}

impl Opts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mobile_to_desktop(self, flag: bool) -> Self {
        Self {
            mobile_to_desktop: flag,
            ..self
        }
    }
}
