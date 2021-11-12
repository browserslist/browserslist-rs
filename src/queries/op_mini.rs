use super::{Distrib, Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(?:operamini|op_mini)\s+all$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct OperaMiniSelector;

impl Selector for OperaMiniSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            Ok(Some(vec![Distrib::new("op_mini", "all")]))
        } else {
            Ok(None)
        }
    }
}
