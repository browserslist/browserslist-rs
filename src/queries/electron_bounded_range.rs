use super::{Distrib, Selector, SelectorResult};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s+(\d+\.\d+)(?:\.\d+)?\s*-\s*(\d+\.\d+)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronBoundedRangeSelector;

impl Selector for ElectronBoundedRangeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let from: f32 = cap[1].parse().map_err(Error::ParseVersion)?;
            let to: f32 = cap[2].parse().map_err(Error::ParseVersion)?;

            if ELECTRON_VERSIONS
                .iter()
                .all(|(version, _)| *version != from)
            {
                return Err(Error::UnknownElectronVersion(cap[1].to_string()));
            }
            if ELECTRON_VERSIONS.iter().all(|(version, _)| *version != to) {
                return Err(Error::UnknownElectronVersion(cap[2].to_string()));
            }

            let versions = ELECTRON_VERSIONS
                .iter()
                .filter(|(version, _)| from <= *version && *version <= to)
                .map(|(_, version)| Distrib::new("chrome", version))
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

    #[test_case("electron 0.36-1.2"; "basic")]
    #[test_case("Electron 0.37-1.0"; "case insensitive")]
    #[test_case("electron 0.37.5-1.0.3"; "with semver patch version")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "electron 0.1-1.2", Error::UnknownElectronVersion(String::from("0.1"));
        "unknown version 1"
    )]
    #[test_case(
        "electron 0.37-999.0", Error::UnknownElectronVersion(String::from("999.0"));
        "unknown version 2"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
