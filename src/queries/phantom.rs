use super::{Selector, SelectorResult, Version};
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
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            match &cap[1] {
                "1.9" => Ok(Some(vec![Version::new("safari", "5")])),
                "2.1" => Ok(Some(vec![Version::new("safari", "6")])),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }
}
