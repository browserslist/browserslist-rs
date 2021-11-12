use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_BROWSERS},
    error::Error,
    opts::Opts,
};
use chrono::{LocalResult, TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^since\s((\d+)(?:-(\d+)(?:-(\d+))?)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct SinceSelector;

impl Selector for SinceSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let year = cap[2]
                .parse()
                .map_err(|_| Error::InvalidDate(cap[1].to_string()))?;
            let month = match cap.get(3) {
                Some(m) => m
                    .as_str()
                    .parse()
                    .map_err(|_| Error::InvalidDate(cap[1].to_string()))?,
                None => 1,
            };
            let day = match cap.get(4) {
                Some(m) => m
                    .as_str()
                    .parse()
                    .map_err(|_| Error::InvalidDate(cap[1].to_string()))?,
                None => 1,
            };
            let time = match Utc.ymd_opt(year, month, day) {
                LocalResult::Single(date) => date.and_hms(0, 0, 0).timestamp(),
                _ => return Err(Error::InvalidDate(cap[1].to_string())),
            };

            let versions = CANIUSE_LITE_BROWSERS
                .keys()
                .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
                .map(|(name,stat)| {
                    stat.released
                        .iter()
                        .filter(|version| matches!(stat.release_date.get(*version), Some(Some(date)) if *date >= time))
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
