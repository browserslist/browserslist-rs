use super::{Selector, SelectorResult, Version};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(?:firefox|ff|fx)\s+esr$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct FirefoxESRSelector;

impl Selector for FirefoxESRSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if REGEX.is_match(text) {
            Ok(Some(vec![
                Version("firefox", "78"),
                Version("firefox", "91"),
            ]))
        } else {
            Ok(None)
        }
    }
}
