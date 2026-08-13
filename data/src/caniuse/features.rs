use super::{PooledStr, VersionDetail};
use crate::{decode_browser_name, utils::BinMap};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList {
    // The browser's own version list, in release order.
    versions: &'static [VersionDetail],
    // Index of this browser's first flag within FEATURES_STAT_FLAGS.
    base: u32,
}

// ```rust
// static FEATURES: &[(PooledStr, Feature)]; // feature name and browsers list
//
// static FEATURES_STAT_FLAGS: &[u8]; // support flag
// static FEATURES_STAT_BROWSERS: &[u8]; // browser name id
// ```
include!("../generated/caniuse-feature-matching.rs");

// caniuse states every feature for every version of a browser (checked by
// generate-data), so a browser's flags line up with its version list position by
// position, and each browser's run of flags is as long as its version list.
static FEATURES_STAT_FLAG_START: LazyLock<Vec<u32>> = LazyLock::new(|| {
    let mut start = 0;
    FEATURES_STAT_BROWSERS
        .iter()
        .map(|id| {
            let base = start;
            start += browser_of(*id).version_list().len() as u32;
            base
        })
        .collect()
});

pub fn get_feature_stat(name: &str) -> Option<Feature> {
    BinMap(FEATURES).get(name).copied()
}

fn browser_of(id: u8) -> &'static super::BrowserStat {
    super::browser_stat(decode_browser_name(id)).expect("feature refers to unknown browser")
}

fn version_list_at(index: usize) -> VersionList {
    let stat = browser_of(FEATURES_STAT_BROWSERS[index]);
    VersionList {
        versions: stat.version_list(),
        base: FEATURES_STAT_FLAG_START[index],
    }
}

impl Feature {
    pub fn get(&self, browser: &str) -> Option<VersionList> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_BROWSERS[range.clone()]
            .binary_search_by_key(&browser, |&k| decode_browser_name(k))
            .ok()?;
        Some(version_list_at(range.start + index))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let start = self.0 as usize;
        FEATURES_STAT_BROWSERS[start..(self.1 as usize)]
            .iter()
            .enumerate()
            .map(move |(offset, &id)| (decode_browser_name(id), version_list_at(start + offset)))
    }
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        let position = self
            .versions
            .iter()
            .position(|probe| probe.version() == version)?;
        Some(FEATURES_STAT_FLAGS[self.base as usize + position])
    }
}
