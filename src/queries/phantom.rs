use super::Selector;
use crate::{error::Error, opts::Opts};
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
    fn select(&self, text: &str, _: &Opts) -> Result<Option<Vec<String>>, Error> {
        if let Some(cap) = REGEX.captures(text) {
            match &cap[1] {
                "1.9" => Ok(Some(vec!["safari 5".to_string()])),
                "2.1" => Ok(Some(vec!["safari 6".to_string()])),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }
}
