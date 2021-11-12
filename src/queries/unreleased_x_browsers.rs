use super::{Distrib, Selector, SelectorResult};
use crate::{data::caniuse::get_browser_stat, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^unreleased\s+(\w+)\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct UnreleasedXBrowsersSelector;

impl Selector for UnreleasedXBrowsersSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let name = &cap[1];
            let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
                .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
            let versions = stat
                .versions
                .iter()
                .filter(|version| !stat.released.contains(version))
                .map(|version| Distrib::new(name, version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
