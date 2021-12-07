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
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let count = cap[1].parse().map_err(Error::ParseVersionsCount)?;
        let name = &cap[2];

        let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
            .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
        let count = if should_filter_android(name, opts.mobile_to_desktop) {
            count_android_filter(count, opts.mobile_to_desktop)
        } else {
            count
        };

        let versions = stat
            .version_list
            .iter()
            .filter(|version| version.release_date.is_some())
            .rev()
            .take(count)
            .map(|version| Distrib::new(name, &*version.version))
            .collect();
        Ok(Some(versions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 ie versions"; "basic")]
    #[test_case("last 2 safari versions"; "do not include unreleased versions")]
    #[test_case("last 1 ie version"; "support pluralization")]
    #[test_case("last 01 Explorer version"; "alias")]
    #[test_case("Last 01 IE Version"; "case insensitive")]
    #[test_case("last 4 android versions"; "android 1")]
    #[test_case("last 31 android versions"; "android 2")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
