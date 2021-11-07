use super::Selector;
use crate::{error::Error, opts::Opts, resolve};

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select(&self, text: &str, opts: &Opts) -> Result<Option<Vec<String>>, Error> {
        if text.eq_ignore_ascii_case("defaults") {
            resolve(
                &["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"],
                opts,
            )
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
