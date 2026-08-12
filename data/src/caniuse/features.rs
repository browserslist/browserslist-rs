use super::PooledStr;
use crate::{
    decode_browser_name,
    utils::{BinMap, undelta},
};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList(u32, u32);

// ```rust
// The ranges below tile their arrays end to end, so only widths are bundled and the
// starts are a prefix sum, rebuilt on first use.
//
// static FEATURES_KEY_DELTA: &[u32]; // feature name
// static FEATURES_WIDTH: &[u8]; // browsers list width
//
// static FEATURES_STAT_VERSION_STORE: &[u32]; // version string
// static FEATURES_STAT_VERSION_WIDTH: &[u8]; // version range width
//
// static FEATURES_STAT_FLAGS: &[u8]; // support flag, two bits each
// static FEATURES_STAT_BROWSERS: &[u8]; // browser name id
// ```
include!("../generated/caniuse-feature-matching.rs");

static FEATURES: LazyLock<Vec<(PooledStr, Feature)>> = LazyLock::new(|| {
    let mut start = 0;
    undelta(FEATURES_KEY_DELTA)
        .zip(FEATURES_WIDTH)
        .map(|(key, width)| {
            let end = start + u32::from(*width);
            let feature = Feature(start, end);
            start = end;
            (PooledStr(key), feature)
        })
        .collect()
});

static FEATURES_STAT_VERSION_INDEX: LazyLock<Vec<(u32, u32)>> = LazyLock::new(|| {
    let mut start = 0;
    FEATURES_STAT_VERSION_WIDTH
        .iter()
        .map(|width| {
            let end = start + u32::from(*width);
            let range = (start, end);
            start = end;
            range
        })
        .collect()
});

pub fn get_feature_stat(name: &str) -> Option<Feature> {
    BinMap(&FEATURES).get(name).copied()
}

impl Feature {
    pub fn get(&self, browser: &str) -> Option<VersionList> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_BROWSERS[range.clone()]
            .binary_search_by_key(&browser, |&k| decode_browser_name(k))
            .ok()?;
        let list = FEATURES_STAT_VERSION_INDEX[range][index];
        Some(VersionList(list.0, list.1))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let range = (self.0 as usize)..(self.1 as usize);
        FEATURES_STAT_BROWSERS[range.clone()]
            .iter()
            .zip(&FEATURES_STAT_VERSION_INDEX[range])
            .map(|(&name, &list)| (decode_browser_name(name), VersionList(list.0, list.1)))
    }
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_VERSION_STORE[range.clone()]
            .binary_search_by_key(&version, |s| PooledStr(*s).as_str())
            .ok()?;
        // Two bits per flag.
        let index = range.start + index;
        Some((FEATURES_STAT_FLAGS[index / 4] >> (2 * (index % 4))) & 3)
    }
}
