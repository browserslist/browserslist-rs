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

            let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop).ok_or_else(|| {
                if name.eq_ignore_ascii_case("node") {
                    Error::UnknownNodejsVersion(format!("{} - {}", from, to))
                } else if name.eq_ignore_ascii_case("electron") {
                    Error::UnknownElectronVersion(format!("{} - {}", from, to))
                } else {
                    Error::BrowserNotFound(name.to_string())
                }
            })?;
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
    use crate::test::{run_compare, should_failed};
    use test_case::test_case;

    #[test_case("ie 8-10"; "basic")]
    #[test_case("ie 8   -  10"; "more spaces")]
    #[test_case("ie 1-12"; "out of range")]
    #[test_case("android 4.3-37"; "android")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case("and_chr 52-53"; "chrome")]
    #[test_case("android 4.4-38"; "android")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, Opts::new().mobile_to_desktop(true));
    }

    #[test_case(
        "unknown 4-7", Error::BrowserNotFound(String::from("unknown"));
        "unknown browser"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
