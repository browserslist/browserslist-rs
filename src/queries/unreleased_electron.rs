use super::{Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^unreleased\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct UnreleasedElectronSelector;

impl Selector for UnreleasedElectronSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if REGEX.is_match(text) {
            Ok(Some(vec![]))
        } else {
            Ok(None)
        }
    }
}
