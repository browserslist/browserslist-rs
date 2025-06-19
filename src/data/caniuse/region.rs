use super::PooledStr;
use crate::data::{
    decode_browser_name,
    utils::{BinMap, U32},
};

#[derive(Clone, Copy)]
pub struct RegionData(u32, u32);

// ```rust
// static REGIONS: &[(PooledStr, RegionData)]; // region name and region data
//
// static REGIONS_BROWSERS: &[u8]; // browser name id
// static REGIONS_VERSIONS: &[U32]; // version string
// static REGIONS_USAGES: &[U32]; // browser usage (f32)
// ```
include!("../../generated/caniuse-region-matching.rs");

pub(crate) fn get_usage_by_region(region: &str) -> Option<RegionData> {
    BinMap(REGIONS).get(region).copied()
}

impl RegionData {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str, f32)> {
        let range = (self.0 as usize)..(self.1 as usize);

        REGIONS_BROWSERS[range.clone()]
            .iter()
            .zip(&REGIONS_VERSIONS[range.clone()])
            .zip(&REGIONS_USAGES[range])
            .map(|((browser, version), usage)| {
                (
                    decode_browser_name(*browser),
                    PooledStr(version.get()).as_str(),
                    f32::from_bits(usage.get()),
                )
            })
    }
}
