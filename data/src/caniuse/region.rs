use super::PooledStr;
use crate::{decode_browser_name, utils::BinMap};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct RegionData(u32, u32);

// ```rust
// static REGIONS_KEY/START/END: parallel region name and data ranges
//
// static REGIONS_BROWSERS: &[u8]; // browser name id
// static REGIONS_VERSIONS: &[u32]; // version string
// static REGIONS_USAGES: &[u32]; // browser usage (f32)
// ```
include!("../generated/caniuse-region-matching.rs");

static REGIONS: LazyLock<Vec<(PooledStr, RegionData)>> = LazyLock::new(|| {
    REGIONS_KEY
        .iter()
        .zip(&*REGIONS_START)
        .zip(&*REGIONS_END)
        .map(|((key, start), end)| (PooledStr(*key), RegionData(*start, *end)))
        .collect()
});

pub fn get_usage_by_region(region: &str) -> Option<RegionData> {
    BinMap(&REGIONS).get(region).copied()
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
                    PooledStr(*version).as_str(),
                    f32::from_bits(*usage),
                )
            })
    }
}
