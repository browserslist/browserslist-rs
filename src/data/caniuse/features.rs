use super::{ BinMap, PooledStr };
use crate::data::decode_browser_name;
use crate::data::utils::PairU32;

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList(PairU32);

include!("../../generated/caniuse-feature-matching.rs");

static FEATURES_STAT_FLAGS: &[u8] = include_bytes!("../../generated/caniuse-feature-flags.bin");
static FEATURES_STAT_BROWSERS: &[u8] = include_bytes!("../../generated/caniuse-feature-browsers.bin");

pub(crate) fn get_feature_stat(name: &str) -> Option<Feature> {
    BinMap(FEATURES).get(name).copied()
}

impl Feature {
    pub fn get(&self, browser: &str) -> Option<VersionList> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_BROWSERS[range.clone()].binary_search_by_key(
            &browser,
            |&k| decode_browser_name(k)
        )
            .ok()?;
        let pair = FEATURES_STAT_VERSION_INDEX[range.clone()][index];
        Some(VersionList(pair))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let range = (self.0 as usize)..(self.1 as usize);
        FEATURES_STAT_BROWSERS[range.clone()]
            .iter()
            .zip(&FEATURES_STAT_VERSION_INDEX[range])
            .map(|(&name, &list)| (decode_browser_name(name), VersionList(list)))
    }
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        let range = (self.0.0.get() as usize)..(self.0.1.get() as usize);
        let index = FEATURES_STAT_VERSION_STORE[range.clone()]
            .binary_search_by_key(&version, |s| PooledStr(s.0.get(), s.1.get()).as_str())
            .ok()?;
        Some(FEATURES_STAT_FLAGS[range][index])
    }
}
