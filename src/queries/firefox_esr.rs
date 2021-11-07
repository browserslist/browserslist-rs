use super::Selector;
use crate::{error::Error, opts::Opts};
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
    fn select(&self, text: &str, _: &Opts) -> Result<Option<Vec<String>>, Error> {
        if REGEX.is_match(text) {
            Ok(Some(vec!["firefox 78".into(), "firefox 91".into()]))
        } else {
            Ok(None)
        }
    }
}
