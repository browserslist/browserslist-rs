use super::{count_android_filter, should_filter_android, Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_BROWSERS},
    error::Error,
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNBrowsersSelector;

impl Selector for LastNBrowsersSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        let count: usize = match REGEX.captures(text) {
            Some(cap) => cap[1].parse().map_err(Error::ParseVersionsCount)?,
            None => return Ok(None),
        };

        let versions = CANIUSE_LITE_BROWSERS
            .keys()
            .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
            .map(|(name, stat)| {
                let count = if should_filter_android(name, opts.mobile_to_desktop) {
                    count_android_filter(count, opts.mobile_to_desktop)
                } else {
                    count
                };

                stat.released
                    .iter()
                    .rev()
                    .take(count)
                    .map(move |version| Distrib::new(name, version))
            })
            .flatten()
            .collect();

        Ok(Some(versions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 versions"; "basic")]
    #[test_case("last 31 versions"; "android")]
    #[test_case("last 1 version"; "support pluralization")]
    #[test_case("Last 02 Versions"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
