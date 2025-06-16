use ahash::AHashMap;
use std::borrow::Borrow;
use std::fmt;
use std::{borrow::Cow, sync::LazyLock};

pub(crate) mod features;
pub(crate) mod region;

use crate::data::BinaryMap;

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;
pub const OP_MOB_BLINK_FIRST: u32 = 14;

#[derive(Clone, Debug)]
pub struct BrowserStat(u32, u32);

#[derive(Clone, Copy)]
pub struct PooledStr(u32, u32);

#[derive(Clone, Debug)]
pub struct VersionDetail {
    pub version: PooledStr,
    pub release_date: i64,
    // Use bool instead of Option to use pad space
    pub released: bool,
    pub global_usage: f32,
}

include!("../generated/caniuse-browsers.rs");

pub static CANIUSE_BROWSERS: BinaryMap<PooledStr, BrowserStat> = BinaryMap(BROWSERS_STATS);

pub static CANIUSE_GLOBAL_USAGE: &[(PooledStr, PooledStr, f32)] =
    include!("../generated/caniuse-global-usage.rs");

pub static BROWSER_VERSION_ALIASES: LazyLock<
    AHashMap<&'static str, AHashMap<&'static str, &'static str>>,
> = LazyLock::new(|| {
    let mut aliases = CANIUSE_BROWSERS
        .iter()
        .filter_map(|(name, stat)| {
            let name = name.as_str();
            let aliases = stat
                .version_list()
                .iter()
                .filter_map(|version| {
                    version
                        .version
                        .as_str()
                        .split_once('-')
                        .map(|(bottom, top)| (bottom, top, version.version))
                })
                .fold(
                    AHashMap::<&str, &str>::new(),
                    move |mut aliases, (bottom, top, version)| {
                        let _ = aliases.insert(bottom, version.as_str());
                        let _ = aliases.insert(top, version.as_str());
                        aliases
                    },
                );
            if aliases.is_empty() {
                None
            } else {
                Some((name, aliases))
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

static ANDROID_TO_DESKTOP: LazyLock<Vec<VersionDetail>> = LazyLock::new(|| {
    let chrome = CANIUSE_BROWSERS.get("chrome").unwrap();
    let android = CANIUSE_BROWSERS.get("android").unwrap();

    let chrome_point = chrome
        .version_list()
        .binary_search_by_key(&(ANDROID_EVERGREEN_FIRST as usize), |probe| {
            probe.version.as_str().parse::<usize>().unwrap()
        })
        .unwrap();

    android
        .version_list()
        .iter()
        .filter(|version| {
            let version = version.version.as_str();
            version.starts_with("2.")
                || version.starts_with("3.")
                || version.starts_with("4.")
                || version == "3"
                || version == "4"
        })
        .chain(chrome.version_list().iter().skip(chrome_point))
        .cloned()
        .collect()
});

pub fn get_browser_stat(
    name: &str,
    mobile_to_desktop: bool,
) -> Option<(&'static str, &'static [VersionDetail])> {
    let name = if name.bytes().all(|b| b.is_ascii_lowercase()) {
        Cow::from(name)
    } else {
        Cow::from(name.to_ascii_lowercase())
    };
    let name = get_browser_alias(&name);

    if mobile_to_desktop {
        if let Some(desktop_name) = to_desktop_name(name) {
            match name {
                "android" => Some(("android", &*ANDROID_TO_DESKTOP)),
                "op_mob" => {
                    let stat = CANIUSE_BROWSERS.get("opera").unwrap();
                    Some(("op_mob", stat.version_list()))
                }
                _ => CANIUSE_BROWSERS.get(desktop_name).map(|stat| {
                    (
                        get_mobile_by_desktop_name(desktop_name),
                        stat.version_list(),
                    )
                }),
            }
        } else {
            CANIUSE_BROWSERS
                .get_key_value(name)
                .map(|(k, v)| (k.as_str(), v.version_list()))
        }
    } else {
        CANIUSE_BROWSERS
            .get_key_value(name)
            .map(|(k, v)| (k.as_str(), v.version_list()))
    }
}

pub fn iter_browser_stat(
    mobile_to_desktop: bool,
) -> impl Iterator<Item = (&'static str, &'static [VersionDetail])> {
    CANIUSE_BROWSERS.iter().filter_map(move |(name, stat)| {
        match (
            mobile_to_desktop,
            to_desktop_name(name.as_str()),
            name.as_str(),
        ) {
            (false, _, _) | (true, None, _) => Some((name.as_str(), stat.version_list())),
            (true, Some(_), "android") => Some(("android", &*ANDROID_TO_DESKTOP)),
            (true, Some(_), "op_mob") => {
                let stat = CANIUSE_BROWSERS.get("opera").unwrap();
                Some(("op_mob", stat.version_list()))
            }
            (true, Some(desktop_name), _) => CANIUSE_BROWSERS.get(desktop_name).map(|stat| {
                (
                    get_mobile_by_desktop_name(desktop_name),
                    stat.version_list(),
                )
            }),
        }
    })
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
    name: &'a str,
    version_list: &'static [VersionDetail],
    version: &'a str,
) -> Option<&'a str> {
    if version_list.iter().any(|v| v.version.as_str() == version) {
        Some(version)
    } else if let Some(version) = BROWSER_VERSION_ALIASES
        .get(name)
        .and_then(|aliases| aliases.get(version))
    {
        Some(version)
    } else if version_list.len() == 1 {
        version_list.first().map(|s| s.version.as_str())
    } else {
        None
    }
}

impl BrowserStat {
    pub fn version_list(&self) -> &'static [VersionDetail] {
        let range = (self.0 as usize)..(self.1 as usize);
        &VERSION_LIST[range]
    }
}

impl PooledStr {
    pub fn as_str(&self) -> &'static str {
        let range = (self.0 as usize)..(self.1 as usize);
        &CANIUSE_STRPOOL[range]
    }
}

impl Borrow<str> for PooledStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PooledStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for PooledStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
