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
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
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
