use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub(super) struct BrowserStat {
    pub(super) name: String,
    pub(super) versions: Vec<String>,
    pub(super) released: Vec<String>,
    #[serde(rename = "releaseDate")]
    pub(super) release_date: HashMap<String, Option<u32>>,
}

pub(super) type CaniuseData = HashMap<String, BrowserStat>;

pub(super) static CANIUSE_LITE_BROWSERS: Lazy<CaniuseData> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-lite-browsers.json"
    )))
    .unwrap()
});

pub(super) static CANIUSE_LITE_USAGE: Lazy<HashMap<String, f32>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-lite-usage.json"
    )))
    .unwrap()
});
