use super::PooledStr;
use crate::{
    decode_browser_name,
    utils::{BinMap, U32},
};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList(u32, u32);

// ```rust
// static FEATURES_KEY: &[PooledStr]; // feature name
// static FEATURES_START: &[u32]; // browsers list start
// static FEATURES_END: &[u32]; // browsers list end
//
// static FEATURES_STAT_VERSION_STORE: &[U32]; // version string
// static FEATURES_STAT_VERSION_START: &[U32]; // version range start
// static FEATURES_STAT_VERSION_END: &[U32]; // version range end
//
// static FEATURES_STAT_FLAGS: &[u8]; // support flag
// static FEATURES_STAT_BROWSERS: &[u8]; // browser name id
// ```
include!("../generated/caniuse-feature-matching.rs");

static FEATURES: LazyLock<Vec<(PooledStr, Feature)>> = LazyLock::new(|| {
    (0..FEATURES_KEY.len())
        .map(|index| {
            (
                FEATURES_KEY[index],
                Feature(FEATURES_START[index], FEATURES_END[index]),
            )
        })
        .collect()
});

static FEATURES_STAT_VERSION_INDEX: LazyLock<Vec<(u32, u32)>> = LazyLock::new(|| {
    (0..FEATURES_STAT_VERSION_START.len())
        .map(|index| {
            (
                FEATURES_STAT_VERSION_START[index].get(),
                FEATURES_STAT_VERSION_END[index].get(),
            )
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
            .binary_search_by_key(&version, |s| PooledStr(s.get()).as_str())
            .ok()?;
        Some(FEATURES_STAT_FLAGS[range][index])
    }
}
