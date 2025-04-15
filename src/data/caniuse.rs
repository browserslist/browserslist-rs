use ahash::AHashMap;
use std::{borrow::Cow, sync::LazyLock};

pub(crate) mod features;
pub(crate) mod region;

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;
pub const OP_MOB_BLINK_FIRST: u32 = 14;

#[derive(Clone, Debug)]
pub struct BrowserStat {
    name: &'static str,
    pub version_list: Vec<VersionDetail>,
}

#[derive(Clone, Debug)]
pub struct VersionDetail {
    pub version: &'static str,
    pub global_usage: f32,
    pub release_date: Option<i64>,
}

pub type CaniuseData = AHashMap<&'static str, BrowserStat>;

pub static CANIUSE_BROWSERS: LazyLock<CaniuseData> =
    LazyLock::new(|| include!("../generated/caniuse-browsers.rs"));

pub static CANIUSE_GLOBAL_USAGE: &[(&'static str, &'static str, f32)] =
    include!("../generated/caniuse-global-usage.rs");

pub static BROWSER_VERSION_ALIASES: LazyLock<
    AHashMap<&'static str, AHashMap<&'static str, &'static str>>,
> = LazyLock::new(|| {
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
                        .map(|(bottom, top)| (bottom, top, version.version))
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
                Some((*name, aliases))
            }
        })
        .collect::<AHashMap<&'static str, _>>();
    let _ = aliases.insert("op_mob", {
        let mut aliases = AHashMap::new();
        let _ = aliases.insert("59", "58");
        aliases
    });
    aliases
});

static ANDROID_TO_DESKTOP: LazyLock<BrowserStat> = LazyLock::new(|| {
    let chrome = CANIUSE_BROWSERS.get("chrome").unwrap();
    let mut android = CANIUSE_BROWSERS.get("android").unwrap().clone();

    android.version_list = android
        .version_list
        .into_iter()
        .filter(|version| {
            let version = version.version;
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
                    chrome
                        .version_list
                        .iter()
                        .position(|version| {
                            version.version.parse::<usize>().unwrap()
                                == ANDROID_EVERGREEN_FIRST as usize
                        })
                        .unwrap(),
                )
                .cloned(),
        )
        .collect();

    android
});

static OPERA_MOBILE_TO_DESKTOP: LazyLock<BrowserStat> =
    LazyLock::new(|| CANIUSE_BROWSERS.get("opera").unwrap().clone());

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
                    .get(desktop_name)
                    .map(|stat| (get_mobile_by_desktop_name(desktop_name), stat)),
            }
        } else {
            CANIUSE_BROWSERS.get(name).map(|stat| (stat.name, stat))
        }
    } else {
        CANIUSE_BROWSERS.get(name).map(|stat| (stat.name, stat))
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

pub(crate) fn to_desktop_name(name: &str) -> Option<&'static str> {
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
        stat.version_list.first().map(|s| s.version)
    } else {
        None
    }
}
