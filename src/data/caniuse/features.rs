use ahash::AHashMap;
use indexmap::IndexMap;

type Feature = AHashMap<&'static str, IndexMap<&'static str, u8>>;

pub(crate) fn get_feature_stat(name: &str) -> Option<&'static Feature> {
    include!("../../generated/caniuse-feature-matching.rs")
}
