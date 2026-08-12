use super::PooledStr;
use crate::{
    decode_browser_name,
    utils::{BinMap, undelta},
};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct RegionData(u32, u32);

// ```rust
// The regions tile the data arrays end to end, so only widths are bundled and the
// starts are a prefix sum, rebuilt on first use.
//
// static REGIONS_KEY_DELTA: &[u32]; // region name
// static REGIONS_WIDTH: &[u16]; // region data width
//
// static REGIONS_BROWSERS: &[u8]; // browser name id
// static REGIONS_VERSIONS: &[u32]; // version string
// static REGIONS_USAGES: &[u32]; // browser usage, in hundred-thousandths of a percent
// ```
include!("../generated/caniuse-region-matching.rs");

static REGIONS: LazyLock<Vec<(PooledStr, RegionData)>> = LazyLock::new(|| {
    let mut start = 0;
    undelta(REGIONS_KEY_DELTA)
        .zip(REGIONS_WIDTH)
        .map(|(key, width)| {
            let end = start + u32::from(*width);
            let region = RegionData(start, end);
            start = end;
            (PooledStr(key), region)
        })
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
                    super::per_100k(*usage),
                )
            })
    }
}
