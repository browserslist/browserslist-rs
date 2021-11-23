use super::{Distrib, Selector, SelectorResult};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s*([<>]=?)\s*(\d+\.\d+)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronUnboundedRangeSelector;

impl Selector for ElectronUnboundedRangeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let sign = &cap[1];
            let version: f32 = cap[2].parse().map_err(Error::ParseVersion)?;

            let versions = ELECTRON_VERSIONS
                .iter()
                .filter(|(electron_version, _)| match sign {
                    ">" => *electron_version > version,
                    "<" => *electron_version < version,
                    "<=" => *electron_version <= version,
                    _ => *electron_version >= version,
                })
                .map(|(_, chromium_version)| Distrib::new("chrome", chromium_version))
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

    #[test_case("electron <= 0.21"; "basic")]
    #[test_case("Electron < 0.21"; "case insensitive")]
    #[test_case("Electron < 0.21.5"; "with semver patch version")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "electron < 8.a", Error::UnknownQuery(String::from("electron < 8.a"));
        "malformed version 1"
    )]
    #[test_case(
        "electron >= 1.1.1.1", Error::UnknownElectronVersion(String::from("1.1.1.1"));
        "malformed version 2"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
