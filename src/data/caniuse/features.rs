use super::BrowserNameAtom;

type Feature = Vec<(BrowserNameAtom, &'static str)>;

pub(crate) fn get_feature_stat(name: &str) -> Option<&'static Feature> {
    include!(concat!(env!("OUT_DIR"), "/caniuse-feature-matching.rs"))
}
