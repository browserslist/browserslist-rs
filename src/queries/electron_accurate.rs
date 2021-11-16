use super::{Distrib, Selector, SelectorResult};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s+(\d+(?:\.\d+)?)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronAccurateSelector;

impl Selector for ElectronAccurateSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let version: f32 = cap[1].parse().map_err(Error::ParseVersion)?;

            let versions = ELECTRON_VERSIONS
                .iter()
                .find(|(electron_version, _)| *electron_version == version)
                .map(|(_, chrome_version)| vec![Distrib::new("chrome", chrome_version)])
                .ok_or_else(|| Error::UnknownElectronVersion(version.to_string()))?;
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

    #[test_case("electron 1.1"; "basic")]
    #[test_case("electron 4.0.4"; "with semver patch version")]
    #[test_case("Electron 1.1"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "electron 0.19", Error::UnknownElectronVersion(String::from("0.19"));
        "unknown version"
    )]
    #[test_case(
        "electron 8.a", Error::UnknownQuery(String::from("electron 8.a"));
        "malformed version 1"
    )]
    #[test_case(
        "electron 1.1.1.1", Error::UnknownElectronVersion(String::from("1.1.1.1"));
        "malformed version 2"
    )]
    #[test_case(
        "electron 7.01", Error::UnknownElectronVersion(String::from("7.01"));
        "malformed version 3"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
