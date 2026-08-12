use super::PooledStr;
use crate::{
    blob::Blob,
    decode_browser_name,
    utils::{BinMap, undelta},
};
use std::{cmp::Ordering, sync::LazyLock};

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList(u32, u32);

// ```rust
// The ranges below tile their arrays end to end, so only widths are bundled and the
// starts are a prefix sum, rebuilt on first use.
//
// static FEATURES_KEY_DELTA: &[u32]; // feature name
// static FEATURES_WIDTH: Blob; // browsers list width
//
// static FEATURES_STAT_VERSION_LO: Blob; // version, low byte of a VERSION_TABLE index
// static FEATURES_STAT_VERSION_HI: Blob; // version, high byte
// static FEATURES_STAT_VERSION_WIDTH: Blob; // version range width
//
// static FEATURES_STAT_FLAGS: Blob; // support flag, two bits each
// static FEATURES_STAT_BROWSERS: Blob; // browser name id
// ```
include!("../generated/caniuse-feature-matching.rs");

static FEATURES: LazyLock<Vec<(PooledStr, Feature)>> = LazyLock::new(|| {
    let mut start = 0;
    undelta(FEATURES_KEY_DELTA)
        .zip(FEATURES_WIDTH.get())
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
        .get()
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
        let index = FEATURES_STAT_BROWSERS.get()[range.clone()]
            .binary_search_by_key(&browser, |&k| decode_browser_name(k))
            .ok()?;
        let list = FEATURES_STAT_VERSION_INDEX[range][index];
        Some(VersionList(list.0, list.1))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let range = (self.0 as usize)..(self.1 as usize);
        FEATURES_STAT_BROWSERS.get()[range.clone()]
            .iter()
            .zip(&FEATURES_STAT_VERSION_INDEX[range])
            .map(|(&name, &list)| (decode_browser_name(name), VersionList(list.0, list.1)))
    }
}

/// The version at `index`, resolved through the shared table. Kept as a lookup rather
/// than a materialized array so that neither the indices nor the strings are copied to
/// the heap.
fn version_at(index: usize) -> &'static str {
    let table_index = u16::from_le_bytes([
        FEATURES_STAT_VERSION_LO.get()[index],
        FEATURES_STAT_VERSION_HI.get()[index],
    ]);
    super::version_in_table(table_index)
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        // The range is ordered by version string, so it is binary searched by hand:
        // the versions live behind an index and are not a slice to search over.
        let (mut low, mut high) = (self.0 as usize, self.1 as usize);
        while low < high {
            let mid = low + (high - low) / 2;
            match version_at(mid).cmp(version) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                // Two bits per flag.
                Ordering::Equal => {
                    return Some((FEATURES_STAT_FLAGS.get()[mid / 4] >> (2 * (mid % 4))) & 3);
                }
            }
        }
        None
    }
}
