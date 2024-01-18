use super::browser_name::BrowserNameAtom;
use ahash::AHashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::borrow::Cow;

pub(crate) mod features;
pub(crate) mod region;

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;
pub const OP_MOB_BLINK_FIRST: u32 = 14;

#[derive(Clone, Debug, Deserialize)]
pub struct BrowserStat {
    name: BrowserNameAtom,
    pub version_list: Vec<VersionDetail>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionDetail {
    pub version: String,
    pub global_usage: f32,
    pub release_date: Option<i64>,
}

pub type CaniuseData = AHashMap<BrowserNameAtom, BrowserStat>;

pub static CANIUSE_BROWSERS: Lazy<CaniuseData> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-browsers.json"
    )))
    .unwrap()
});

pub static CANIUSE_GLOBAL_USAGE: Lazy<Vec<(BrowserNameAtom, String, f32)>> = Lazy::new(|| {
    serde_json::from_str::<Vec<(u8, String, f32)>>(include_str!(concat!(
        env!("OUT_DIR"),
        "/caniuse-global-usage.json"
    )))
    .unwrap()
    .into_iter()
    .map(|(browser, version, usage)| {
        (
            super::browser_name::decode_browser_name(browser),
            version,
            usage,
        )
    })
    .collect()
});

pub static BROWSER_VERSION_ALIASES: Lazy<
    AHashMap<BrowserNameAtom, AHashMap<&'static str, &'static str>>,
> = Lazy::new(|| {
    let mut aliases = CANIUSE_BROWSERS
        .iter()
        .filter_map(|(name, stat)| {
            let aliases = stat
                .version_list
                .iter()
                .filter_map(|version| {
                    version
                        .version
                        .split_once('-')
                        .map(|(bottom, top)| (bottom, top, version.version.as_str()))
                })
                .fold(
                    AHashMap::<&str, &str>::new(),
                    move |mut aliases, (bottom, top, version)| {
                        let _ = aliases.insert(bottom, version);
                        let _ = aliases.insert(top, version);
                        aliases
                    },
                );
            if aliases.is_empty() {
                None
            } else {
                Some((name.clone(), aliases))
            }
        })
        .collect::<AHashMap<BrowserNameAtom, _>>();
    let _ = aliases.insert("op_mob".into(), {
        let mut aliases = AHashMap::new();
        let _ = aliases.insert("59", "58");
        aliases
    });
    aliases
});

static ANDROID_TO_DESKTOP: Lazy<BrowserStat> = Lazy::new(|| {
    let chrome = CANIUSE_BROWSERS.get(&"chrome".into()).unwrap();
    let mut android = CANIUSE_BROWSERS.get(&"android".into()).unwrap().clone();

    android.version_list = android
        .version_list
        .into_iter()
        .filter(|version| {
            let version = &version.version;
            version.starts_with("2.")
                || version.starts_with("3.")
                || version.starts_with("4.")
                || version == "3"
                || version == "4"
        })
        .chain(
            chrome
                .version_list
                .iter()
                .skip(
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
                )
                .cloned(),
        )
        .collect();

    android
});

static OPERA_MOBILE_TO_DESKTOP: Lazy<BrowserStat> =
    Lazy::new(|| CANIUSE_BROWSERS.get(&"opera".into()).unwrap().clone());

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
                _ => CANIUSE_BROWSERS
                    .get(&desktop_name.into())
                    .map(|stat| (get_mobile_by_desktop_name(desktop_name), stat)),
            }
        } else {
            CANIUSE_BROWSERS
                .get(&name.into())
                .map(|stat| (&*stat.name, stat))
        }
    } else {
        CANIUSE_BROWSERS
            .get(&name.into())
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
    } else if let Some(version) = BROWSER_VERSION_ALIASES
        .get(&stat.name)
        .and_then(|aliases| aliases.get(version))
    {
        Some(version)
    } else if stat.version_list.len() == 1 {
        stat.version_list.first().map(|s| s.version.as_str())
    } else {
        None
    }
}
