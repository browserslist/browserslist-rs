use super::{Distrib, Selector, SelectorResult};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+electron\s+major\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNElectronMajorSelector;

impl Selector for LastNElectronMajorSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        let count: usize = match REGEX.captures(text) {
            Some(cap) => cap[1].parse().map_err(Error::ParseVersionsCount)?,
            None => return Ok(None),
        };

        let minimum = ELECTRON_VERSIONS
            .iter()
            .rev()
            .dedup()
            .nth(count - 1)
            .map(|(electron_version, _)| electron_version)
            .unwrap_or(&0.0);

        let versions = ELECTRON_VERSIONS
            .iter()
            .filter(|(electron_version, _)| electron_version >= minimum)
            .rev()
            .map(|(_, chrome_version)| Distrib::new("chrome", chrome_version))
            .collect();

        Ok(Some(versions))
    }
}
