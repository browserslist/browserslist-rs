use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, normalize_version},
    error::Error,
    opts::Opts,
    semver::Version,
};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+)\s+([\d.]+)\s*-\s*([\d.]+)$").unwrap());

pub(super) struct BrowserBoundedRangeSelector;

impl Selector for BrowserBoundedRangeSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let name = &cap[1];
            let from = &cap[2];
            let to = &cap[3];

            let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
                .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
            let from: Version = normalize_version(stat, from)
                .unwrap_or(from)
                .parse()
                .unwrap_or_default();
            let to: Version = normalize_version(stat, to)
                .unwrap_or(to)
                .parse()
                .unwrap_or_default();

            let versions = stat
                .released
                .iter()
                .filter(|version| {
                    let version = version.parse().unwrap_or_default();
                    from <= version && version <= to
                })
                .map(|version| Distrib::new(name, version))
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

    #[test_case("and_chr 52-53"; "chrome 3")]
    #[test_case("android 4.4-38"; "android")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, Opts::new().mobile_to_desktop(true));
    }
}
