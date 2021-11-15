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
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
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
                .map(|(name, stat)| {
                    stat.release_date
                        .iter()
                        .filter(|(_, date)| match date {
                            Some(date) => *date >= time,
                            // This is for matching original "browserslist":
                            // For unreleased browsers like `safari TP`,
                            // its released date value is `null`.
                            // When querying `since 1970`,
                            // its corresponding UNIX timestamp is `0`.
                            // In JavaScript, `null >= 0` will be evaluate to `true`.
                            // Thus, for the query `since 1970`,
                            // unreleased browsers versions will be included,
                            // and here we're behaving as same as "browserslist" in JavaScript.
                            None => time == 0,
                        })
                        .map(|(version, _)| Distrib::new(name, version))
                })
                .flatten()
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("since 2017"; "year only")]
    #[test_case("Since 2017"; "case insensitive")]
    #[test_case("since 2017-02"; "with month")]
    #[test_case("since 2017-02-15"; "with day")]
    #[test_case("since 1970"; "unix timestamp zero")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
