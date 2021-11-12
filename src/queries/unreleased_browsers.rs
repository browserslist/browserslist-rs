use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_BROWSERS},
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^unreleased\s+versions$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct UnreleasedBrowsersSelector;

impl Selector for UnreleasedBrowsersSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            let versions = CANIUSE_LITE_BROWSERS
                .keys()
                .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
                .map(|(name, stat)| {
                    stat.versions
                        .iter()
                        .filter(|version| !stat.released.contains(version))
                        .map(|version| Distrib::new(name, version))
                })
                .flatten()
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
