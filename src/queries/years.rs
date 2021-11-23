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

const ONE_YEAR_IN_SECONDS: f64 = 365.259641 * 24.0 * 60.0 * 60.0;

pub(super) struct YearsSelector;

impl Selector for YearsSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let count: f64 = cap[1].parse().map_err(Error::ParseYearsCount)?;
            let duration = Duration::seconds((count * ONE_YEAR_IN_SECONDS) as i64);
            let time = (Utc::now() - duration).timestamp();

            let versions = CANIUSE_LITE_BROWSERS
                .keys()
                .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
                .map(|(name, stat)| {
                    stat.version_list
                        .iter()
                        .filter(
                            |version| matches!(version.release_date, Some(date) if date >= time),
                        )
                        .map(|version| Distrib::new(name, &*version.version))
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

    #[test_case("last 2 years"; "basic")]
    #[test_case("last 1 year"; "one year")]
    #[test_case("last 1.4 years"; "year fraction")]
    #[test_case("Last 5 Years"; "case insensitive")]
    #[test_case("last    2     years"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
