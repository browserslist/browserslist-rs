use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;

#[derive(Clone, Debug, Deserialize)]
pub struct BrowserStat {
    name: String,
    pub versions: Vec<String>,
    pub released: Vec<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: HashMap<String, Option<i64>>,
}

pub type CaniuseData = HashMap<String, BrowserStat>;

pub static CANIUSE_LITE_BROWSERS: Lazy<CaniuseData> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../data/caniuse-lite-browsers.json")).unwrap()
});

pub static CANIUSE_LITE_USAGE: Lazy<Vec<(String, String, f32)>> =
    Lazy::new(|| serde_json::from_str(include_str!("../../data/caniuse-lite-usage.json")).unwrap());

pub static CANIUSE_LITE_VERSION_ALIASES: Lazy<HashMap<String, HashMap<String, String>>> =
    Lazy::new(|| {
        serde_json::from_str(include_str!("../../data/caniuse-lite-version-aliases.json")).unwrap()
    });

static REGEX_NON_DESKTOP_ANDROID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[2-4]\.|[34]$)").unwrap());

static ANDROID_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let chrome = CANIUSE_LITE_BROWSERS.get("chrome").unwrap();
    let mut android = CANIUSE_LITE_BROWSERS.get("android").unwrap().clone();

    android.released = normalize_android_versions(android.released, &chrome.released);
    android.versions = normalize_android_versions(android.versions, &chrome.versions);

    android
});

fn normalize_android_versions(
    android_versions: Vec<String>,
    chrome_versions: &[String],
) -> Vec<String> {
    android_versions
        .into_iter()
        .filter(|version| REGEX_NON_DESKTOP_ANDROID.is_match(version))
        .chain(chrome_versions.iter().cloned().skip(
            chrome_versions.len()
                - (chrome_versions.last().unwrap().parse::<usize>().unwrap()
                    - (ANDROID_EVERGREEN_FIRST as usize)
                    + 1),
        ))
        .collect()
}

static OPERA_MOBILE_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let mut op_mob = CANIUSE_LITE_BROWSERS.get("opera").unwrap().clone();

    if let Some(v) = op_mob
        .versions
        .iter_mut()
        .find(|version| version.as_str() == "10.0-10.1")
    {
        *v = "10".to_string();
    }

    if let Some(v) = op_mob
        .released
        .iter_mut()
        .find(|version| version.as_str() == "10.0-10.1")
    {
        *v = "10".to_string();
    }

    if let Some(value) = op_mob.release_date.remove("10.0-10.1") {
        op_mob.release_date.insert("10".to_string(), value);
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
    if stat.versions.iter().any(|v| v == version) {
        Some(version)
    } else if let Some(version) = CANIUSE_LITE_VERSION_ALIASES
        .get(&stat.name)
        .and_then(|aliases| aliases.get(version).map(|s| s.as_str()))
    {
        Some(version)
    } else if stat.versions.len() == 1 {
        stat.versions.first().map(|s| s.as_str())
    } else {
        None
    }
}
