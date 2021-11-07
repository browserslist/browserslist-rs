use super::{
    caniuse::{get_browser_stat, CANIUSE_LITE_BROWSERS},
    count_android_filter, should_filter_android, Selector,
};
use crate::{error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNVersionsSelector;

impl Selector for LastNVersionsSelector {
    fn select(&self, text: &str, opts: &Opts) -> Result<Option<Vec<String>>, Error> {
        let count: usize = match REGEX.captures(text) {
            Some(cap) => cap[1].parse().map_err(Error::ParseVersionsCount)?,
            None => return Ok(None),
        };

        let versions = CANIUSE_LITE_BROWSERS
            .keys()
            .filter_map(|name| {
                get_browser_stat(name, opts.mobile_to_desktop).map(|stat| (name, stat))
            })
            .map(|(name, stat)| {
                let count = if should_filter_android(name, opts.mobile_to_desktop) {
                    count_android_filter(count, opts.mobile_to_desktop)
                } else {
                    count
                };

                stat.released.iter().rev().take(count).map(|version| {
                    let mut r = name.clone();
                    r.push(' ');
                    r.push_str(&version);
                    r
                })
            })
            .flatten()
            .collect();

        Ok(Some(versions))
    }
}
