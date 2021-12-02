use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

type Feature = Vec<(&'static str, &'static str)>;

static FEATURES_LIST: Lazy<Vec<&'static str>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-features-list.json"
    )))
    .unwrap()
});

static FEATURES: Lazy<RwLock<AHashMap<String, Arc<Feature>>>> =
    Lazy::new(|| RwLock::new(AHashMap::new()));

pub(crate) fn get_feature_stat(name: &str) -> Option<Arc<Feature>> {
    if !FEATURES_LIST.contains(&name) {
        return None;
    }

    {
        let features = FEATURES.read().unwrap();
        if let Some(stat) = features.get(name) {
            return Some(Arc::clone(stat));
        }
    }

    let mut features = FEATURES.write().unwrap();
    Some(Arc::clone(features.entry(name.into()).or_insert_with_key(
        |name| {
            let name = name.as_str();
            let stat = serde_json::from_str(include!(concat!(
                env!("OUT_DIR"),
                "/caniuse-feature-matching.rs"
            )))
            .unwrap();
            Arc::new(stat)
        },
    )))
}
