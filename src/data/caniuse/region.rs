use super::BrowserNameAtom;

type RegionData = Vec<(BrowserNameAtom, &'static str, f32)>;

pub(crate) fn get_usage_by_region(region: &str) -> Option<&'static RegionData> {
    include!("../../generated/caniuse-region-matching.rs")
}
