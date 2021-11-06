use super::Selector;
use crate::resolve;

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        if text.eq_ignore_ascii_case("defaults") {
            Some(resolve(&[
                "> 0.5%",
                "last 2 versions",
                "Firefox ESR",
                "not dead",
            ]))
        } else {
            None
        }
    }
}
