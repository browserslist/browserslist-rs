use super::Selector;
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s+(\d+\.\d+)(?:\.\d+)?\s*-\s*(\d+\.\d+)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronSelector;

impl Selector for ElectronSelector {
    fn select(&self, text: &str, _: &Opts) -> Result<Option<Vec<String>>, Error> {
        if let Some(cap) = REGEX.captures(text) {
            let from: f32 = cap[1].parse().map_err(Error::ParseVersion)?;
            let to: f32 = cap[2].parse().map_err(Error::ParseVersion)?;

            let versions = ELECTRON_VERSIONS
                .iter()
                .filter(|(version, _)| from <= *version && *version <= to)
                .map(|(_, version)| format!("chrome {}", version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
