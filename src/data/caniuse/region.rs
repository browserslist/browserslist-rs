use ustr::Ustr;

type RegionData = Vec<(Ustr, &'static str, f32)>;

pub(crate) fn get_usage_by_region(region: &str) -> Option<&'static RegionData> {
    include!(concat!(env!("OUT_DIR"), "/caniuse-region-matching.rs"))
}
