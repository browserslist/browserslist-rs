type RegionData = Vec<(&'static str, &'static str, f32)>;

pub(crate) fn get_usage_by_region(region: &str) -> Option<&'static RegionData> {
    include!("../../generated/caniuse-region-matching.rs")
}
