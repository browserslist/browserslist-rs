use super::{PooledStr, VersionDetail};
use crate::{
    blob::Blob,
    decode_browser_name,
    utils::{BinMap, undelta},
};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList {
    // The browser's own version list, in release order, paired with the permutation
    // that puts it in lexicographic order.
    versions: &'static [VersionDetail],
    lex_order: &'static [u8],
    // Index of this browser's first flag within FEATURES_STAT_FLAGS.
    base: u32,
}

// ```rust
// The ranges below tile their arrays end to end, so only widths are bundled and the
// starts are a prefix sum, rebuilt on first use.
//
// static FEATURES_KEY_DELTA: &[u32]; // feature name
// static FEATURES_WIDTH: Blob; // browsers list width
//
// static FEATURES_STAT_VERSION_WIDTH: Blob; // version range width
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

static FEATURES_STAT_FLAG_START: LazyLock<Vec<u32>> = LazyLock::new(|| {
    let mut start = 0;
    FEATURES_STAT_VERSION_WIDTH
        .get()
        .iter()
        .map(|width| {
            let base = start;
            start += u32::from(*width);
            base
        })
        .collect()
});

pub fn get_feature_stat(name: &str) -> Option<Feature> {
    BinMap(&FEATURES).get(name).copied()
}

// caniuse states every feature for every version of a browser (checked by
// generate-data), so a browser's flags line up with its version list position by
// position and no versions are stored on the feature side at all.
fn version_list_at(index: usize, browser: &str) -> VersionList {
    let stat = super::browser_stat(browser).expect("feature refers to unknown browser");
    VersionList {
        versions: stat.version_list(),
        lex_order: stat.lex_order(),
        base: FEATURES_STAT_FLAG_START[index],
    }
}

impl Feature {
    pub fn get(&self, browser: &str) -> Option<VersionList> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_BROWSERS.get()[range.clone()]
            .binary_search_by_key(&browser, |&k| decode_browser_name(k))
            .ok()?;
        Some(version_list_at(range.start + index, browser))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let start = self.0 as usize;
        FEATURES_STAT_BROWSERS.get()[start..(self.1 as usize)]
            .iter()
            .enumerate()
            .map(move |(offset, &id)| {
                let name = decode_browser_name(id);
                (name, version_list_at(start + offset, name))
            })
    }
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        let rank = self
            .lex_order
            .binary_search_by(|&position| {
                self.versions[usize::from(position)].version().cmp(version)
            })
            .ok()?;
        let index = self.base as usize + usize::from(self.lex_order[rank]);
        // Two bits per flag.
        Some((FEATURES_STAT_FLAGS.get()[index / 4] >> (2 * (index % 4))) & 3)
    }
}
