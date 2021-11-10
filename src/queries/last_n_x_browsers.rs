use super::{count_android_filter, should_filter_android, Distrib, Selector, SelectorResult};
use crate::{data::caniuse::get_browser_stat, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+(\w+)\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNXBrowsersSelector;

impl Selector for LastNXBrowsersSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult<'a> {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let count = cap[1].parse().map_err(Error::ParseVersionsCount)?;
        let name = &cap[2];

        let stat = get_browser_stat(name, opts.mobile_to_desktop)
            .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
        let name = stat.name.as_str();
        let count = if should_filter_android(name, opts.mobile_to_desktop) {
            count_android_filter(count, opts.mobile_to_desktop)
        } else {
            count
        };

        let versions = stat
            .released
            .iter()
            .rev()
            .take(count)
            .map(|version| Distrib::new(name, version))
            .collect();
        Ok(Some(versions))
    }
}
