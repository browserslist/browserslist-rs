use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};

pub(super) const ANDROID_EVERGREEN_FIRST: f32 = 37.0;

#[derive(Clone, Deserialize)]
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

pub(super) static CANIUSE_LITE_VERSION_ALIASES: Lazy<HashMap<String, HashMap<String, String>>> =
    Lazy::new(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/caniuse-lite-version-aliases.json"
        )))
        .unwrap()
    });

static REGEX_NON_DESKTOP_ANDROID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[2-4]\.|[34]$)").unwrap());

static ANDROID_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let chrome = CANIUSE_LITE_BROWSERS.get("chrome").unwrap();
    let mut android = CANIUSE_LITE_BROWSERS.get("android").unwrap().clone();

    android.released = android
        .released
        .into_iter()
        .filter(|version| REGEX_NON_DESKTOP_ANDROID.is_match(version))
        .chain(chrome.released.iter().cloned().skip(
            chrome.released.last().unwrap().parse::<usize>().unwrap()
                - (ANDROID_EVERGREEN_FIRST as usize),
        ))
        .collect();
    android.versions = android
        .versions
        .into_iter()
        .filter(|version| REGEX_NON_DESKTOP_ANDROID.is_match(version))
        .chain(chrome.versions.iter().cloned().skip(
            chrome.versions.last().unwrap().parse::<usize>().unwrap()
                - (ANDROID_EVERGREEN_FIRST as usize),
        ))
        .collect();

    android
});

pub(super) fn get_browser_stat(name: &str, mobile_to_desktop: bool) -> Option<&BrowserStat> {
    let name = if name.bytes().all(|b| b.is_ascii_lowercase()) {
        Cow::from(name)
    } else {
        Cow::from(name.to_ascii_lowercase())
    };
    let name = get_browser_alias(&name);

    if mobile_to_desktop {
        if let Some(desktop_name) = to_desktop_name(name) {
            if name == "android" {
                Some(&ANDROID_TO_DESKTOP)
            } else {
                CANIUSE_LITE_BROWSERS.get(desktop_name)
            }
        } else {
            CANIUSE_LITE_BROWSERS.get(name)
        }
    } else {
        CANIUSE_LITE_BROWSERS.get(name)
    }
}

pub fn get_browser_alias(name: &str) -> &str {
    match name {
        "fx" | "ff" => "firefox",
        "ios" => "ios_saf",
        "explorer" => "ie",
        "blackberry" => "bb",
        "explorermobile" => "ie_mob",
        "operamini" => "op_mini",
        "operamobile" => "op_mob",
        "chromeandroid" => "and_chr",
        "firefoxandroid" => "and_ff",
        "ucandroid" => "and_uc",
        "qqandroid" => "and_qq",
        _ => name,
    }
}

pub fn to_desktop_name(name: &str) -> Option<&'static str> {
    match name {
        "and_chr" => Some("chrome"),
        "and_ff" => Some("firefox"),
        "ie_mob" => Some("ie"),
        "op_mob" => Some("opera"),
        "android" => Some("chrome"),
        _ => None,
    }
}
