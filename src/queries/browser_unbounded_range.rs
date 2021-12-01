use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, BROWSER_VERSION_ALIASES},
    error::Error,
    opts::Opts,
    semver::Version,
};
use once_cell::sync::Lazy;
use regex::Regex;
use ustr::Ustr;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+)\s*([<>]=?)\s*([\d.]+)$").unwrap());

pub(super) struct BrowserUnboundedRangeSelector;

impl Selector for BrowserUnboundedRangeSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let name = &cap[1];
        let sign = &cap[2];
        let version = &cap[3];

        let (name, stat) = get_browser_stat(&cap[1], opts.mobile_to_desktop).ok_or_else(|| {
            if name.eq_ignore_ascii_case("node") {
                Error::UnknownNodejsVersion(version.to_string())
            } else if name.eq_ignore_ascii_case("electron") {
                Error::UnknownElectronVersion(version.to_string())
            } else {
                Error::BrowserNotFound(name.to_string())
            }
        })?;
        let version: Version = BROWSER_VERSION_ALIASES
            .get(&Ustr::from(name))
            .and_then(|alias| alias.get(version).map(|s| *s))
            .unwrap_or(version)
            .parse()
            .unwrap_or_default();

        let versions = stat
            .version_list
            .iter()
            .filter(|version| version.release_date.is_some())
            .map(|version| &*version.version)
            .filter(|v| {
                let v: Version = v.parse().unwrap_or_default();
                match sign {
                    ">" => v > version,
                    "<" => v < version,
                    "<=" => v <= version,
                    _ => v >= version,
                }
            })
            .map(|version| Distrib::new(name, version))
            .collect();
        Ok(Some(versions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{run_compare, should_failed};
    use test_case::test_case;

    #[test_case("ie > 9"; "greater")]
    #[test_case("ie >= 10"; "greater or equal")]
    #[test_case("ie < 10"; "less")]
    #[test_case("ie <= 9"; "less or equal")]
    #[test_case("Explorer > 10"; "case insensitive")]
    #[test_case("android >= 4.2"; "android 1")]
    #[test_case("android >= 4.3"; "android 2")]
    #[test_case("ie<=9"; "no spaces")]
    #[test_case("and_qq > 0"; "browser with one version")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case("chromeandroid >= 52 and chromeandroid < 54"; "chrome")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, Opts::new().mobile_to_desktop(true));
    }

    #[test_case(
        "unknow > 10", Error::BrowserNotFound(String::from("unknow"));
        "unknown browser"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
