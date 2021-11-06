use super::Selector;
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(firefox|ff|fx)\s+esr$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct FirefoxESRSelector;

impl Selector for FirefoxESRSelector {
    fn select(&self, text: &str, _: &Opts) -> Option<Vec<String>> {
        REGEX
            .is_match(text)
            .then(|| vec!["firefox 78".into(), "firefox 91".into()])
    }
}
