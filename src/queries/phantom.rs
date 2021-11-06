use super::Selector;
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"phantomjs\s+(1\.9|2\.1)")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct PhantomSelector;

impl Selector for PhantomSelector {
    fn select(&self, text: &str, _: &Opts) -> Option<Vec<String>> {
        let version = REGEX.captures(text)?.get(1)?.as_str();
        match version {
            "1.9" => Some(vec!["safari 5".to_string()]),
            "2.1" => Some(vec!["safari 6".to_string()]),
            _ => unreachable!(),
        }
    }
}
