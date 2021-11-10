use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_BROWSERS},
    error::Error,
    opts::Opts,
};
use chrono::{Duration, Utc};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d*\.?\d+)\s+years?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

const ONE_YEAR_IN_SECONDS: f32 = 365.259641 * 24.0 * 60.0 * 60.0;

pub(super) struct YearsSelector;

impl Selector for YearsSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let count: f32 = cap[1].parse().map_err(Error::ParseYearsCount)?;
            let duration = Duration::seconds((count * ONE_YEAR_IN_SECONDS) as i64);
            let time = (Utc::now() - duration).timestamp();

            let versions = CANIUSE_LITE_BROWSERS
                .keys()
                .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
                .map(|stat| {
                    stat.released
                        .iter()
                        .filter(|version| matches!(stat.release_date.get(*version), Some(Some(date)) if *date >= time))
                        .map(|version| Distrib::new(&stat.name, version))
                })
                .flatten()
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
