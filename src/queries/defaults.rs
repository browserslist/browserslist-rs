use super::Selector;
use crate::{opts::Opts, resolve};

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select(&self, text: &str, opts: &Opts) -> Option<Vec<String>> {
        if text.eq_ignore_ascii_case("defaults") {
            Some(resolve(
                &["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"],
                opts,
            ))
        } else {
            None
        }
    }
}
