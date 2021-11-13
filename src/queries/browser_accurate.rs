use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, normalize_version},
    error::Error,
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(\w+)\s+(tp|[\d.]+)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct BrowserAccurateSelector;

impl Selector for BrowserAccurateSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let name = &cap[1];
            let version = match &cap[2] {
                version if version.eq_ignore_ascii_case("tp") => "TP",
                version => version,
            };

            let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
                .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;

            if let Some(version) = normalize_version(stat, version) {
                Ok(Some(vec![Distrib::new(name, version.to_owned())]))
            } else {
                let version = if version.contains('.') {
                    Cow::Borrowed(version.trim_end_matches(".0"))
                } else {
                    let mut v = version.to_owned();
                    v.push_str(".0");
                    Cow::Owned(v)
                };
                if let Some(version) = normalize_version(stat, &version) {
                    Ok(Some(vec![Distrib::new(name, version.to_owned())]))
                } else if opts.ignore_unknown_versions {
                    Ok(Some(vec![]))
                } else {
                    Err(Error::UnknownBrowserVersion(
                        cap[1].to_string(),
                        cap[2].to_string(),
                    ))
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        resolve,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("ie 10", &Opts::new(); "by name")]
    #[test_case("IE 10", &Opts::new(); "case insensitive")]
    #[test_case("Explorer 10", &Opts::new(); "alias")]
    #[test_case("ios 7.0", &Opts::new(); "work with joined versions 1")]
    #[test_case("ios 7.1", &Opts::new(); "work with joined versions 2")]
    #[test_case("ios 7", &Opts::new(); "allow missing zero 1")]
    #[test_case("ios 8.0", &Opts::new(); "allow missing zero 2")]
    #[test_case("safari tp", &Opts::new(); "safari tp")]
    #[test_case("Safari TP", &Opts::new(); "safari tp case insensitive")]
    #[test_case("and_uc 10", &Opts::new(); "cutted version")]
    fn valid(query: &str, opts: &Opts) {
        run_compare(query, &opts);
    }

    #[test_case(
        "unknown 10", Error::BrowserNotFound(String::from("unknown"));
        "unknown browser"
    )]
    #[test_case(
        "IE 1", Error::UnknownBrowserVersion(String::from("IE"), String::from("1"));
        "unknown version"
    )]
    #[test_case(
        "chrome 70, ie 11, safari 12.2, safari 12",
        Error::UnknownBrowserVersion(String::from("safari"), String::from("12.2"));
        "use correct browser name in error"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }

    #[test]
    fn ignore_unknown_versions() {
        assert_eq!(
            resolve(["IE 1, IE 9"], &Opts::new().ignore_unknown_versions(true)).unwrap()[0],
            Distrib::new("ie", "9")
        );
    }
}
