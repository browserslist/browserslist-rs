use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;

#[derive(Clone, Debug, Deserialize)]
pub struct BrowserStat {
    name: String,
    pub version_list: Vec<VersionDetail>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionDetail {
    pub version: String,
    pub global_usage: f32,
    pub release_date: Option<i64>,
}

pub type CaniuseData = HashMap<String, BrowserStat>;

pub static CANIUSE_LITE_BROWSERS: Lazy<CaniuseData> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-browsers.json"
    )))
    .unwrap()
});

pub static CANIUSE_LITE_USAGE: Lazy<Vec<(String, String, f32)>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-usage.json"
    )))
    .unwrap()
});

pub static CANIUSE_LITE_VERSION_ALIASES: Lazy<HashMap<String, HashMap<String, String>>> =
    Lazy::new(|| {
        serde_json::from_str(include_str!("../../data/caniuse-lite-version-aliases.json")).unwrap()
    });

static REGEX_NON_DESKTOP_ANDROID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[2-4]\.|[34]$)").unwrap());

static ANDROID_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let chrome = CANIUSE_LITE_BROWSERS.get("chrome").unwrap();
    let mut android = CANIUSE_LITE_BROWSERS.get("android").unwrap().clone();

    android.version_list = android
        .version_list
        .into_iter()
        .filter(|version| REGEX_NON_DESKTOP_ANDROID.is_match(&version.version))
        .chain(
            chrome.version_list.iter().cloned().skip(
                chrome.version_list.len()
                    - (chrome
                        .version_list
                        .last()
                        .unwrap()
                        .version
                        .parse::<usize>()
                        .unwrap()
                        - (ANDROID_EVERGREEN_FIRST as usize)
                        + 1),
            ),
        )
        .collect();

    android
});

static OPERA_MOBILE_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let mut op_mob = CANIUSE_LITE_BROWSERS.get("opera").unwrap().clone();

    if let Some(v) = op_mob
        .version_list
        .iter_mut()
        .find(|version| version.version.as_str() == "10.0-10.1")
    {
        v.version = "10".to_string();
    }

    op_mob
});

pub fn get_browser_stat(
    name: &str,
    mobile_to_desktop: bool,
) -> Option<(&'static str, &'static BrowserStat)> {
    let name = if name.bytes().all(|b| b.is_ascii_lowercase()) {
        Cow::from(name)
    } else {
        Cow::from(name.to_ascii_lowercase())
    };
    let name = get_browser_alias(&name);

    if mobile_to_desktop {
        if let Some(desktop_name) = to_desktop_name(name) {
            match name {
                "android" => Some(("android", &ANDROID_TO_DESKTOP)),
                "op_mob" => Some(("op_mob", &OPERA_MOBILE_TO_DESKTOP)),
                _ => CANIUSE_LITE_BROWSERS
                    .get(desktop_name)
                    .map(|stat| (get_mobile_by_desktop_name(desktop_name), stat)),
            }
        } else {
            CANIUSE_LITE_BROWSERS
                .get(name)
                .map(|stat| (&*stat.name, stat))
        }
    } else {
        CANIUSE_LITE_BROWSERS
            .get(name)
            .map(|stat| (&*stat.name, stat))
    }
}

fn get_browser_alias(name: &str) -> &str {
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

fn to_desktop_name(name: &str) -> Option<&'static str> {
    match name {
        "and_chr" | "android" => Some("chrome"),
        "and_ff" => Some("firefox"),
        "ie_mob" => Some("ie"),
        "op_mob" => Some("opera"),
        _ => None,
    }
}

fn get_mobile_by_desktop_name(name: &str) -> &'static str {
    match name {
        "chrome" => "and_chr", // "android" has been handled as a special case
        "firefox" => "and_ff",
        "ie" => "ie_mob",
        "opera" => "op_mob",
        _ => unreachable!(),
    }
}

pub(crate) fn normalize_version<'a>(
    stat: &'static BrowserStat,
    version: &'a str,
) -> Option<&'a str> {
    if stat.version_list.iter().any(|v| v.version == version) {
        Some(version)
    } else if let Some(version) = CANIUSE_LITE_VERSION_ALIASES
        .get(&stat.name)
        .and_then(|aliases| aliases.get(version).map(|s| s.as_str()))
    {
        Some(version)
    } else if stat.version_list.len() == 1 {
        stat.version_list.first().map(|s| s.version.as_str())
    } else {
        None
    }
}
