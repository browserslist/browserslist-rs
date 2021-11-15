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
                .map(|(_, chrome_version)| Distrib::new("chrome", chrome_version))
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

    #[test_case("electron <= 0.21"; "basic")]
    #[test_case("Electron < 0.21"; "case insensitive")]
    #[test_case("Electron < 0.21.5"; "with semver patch version")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
