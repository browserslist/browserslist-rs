use ahash::AHashMap;
use std::{borrow::Cow, sync::LazyLock};

pub mod features;
pub mod region;

use crate::{
    blob::Blob,
    utils::{BinMap, PooledStr, undelta},
};

/// Usage percentages are bundled as thousandths.
fn per_mille(value: u16) -> f32 {
    f32::from(value) / 1000.0
}

/// Region usage percentages are bundled as hundred-thousandths; see `per_100k` in
/// generate-data for why they need the extra range.
pub(crate) fn per_100k(value: u32) -> f32 {
    value as f32 / 100_000.0
}

/// Version strings are interned into one table; every array that refers to a version
/// holds a `u16` index into it, split across a low and a high byte array.
pub(crate) fn version_in_table(index: u16) -> &'static str {
    VERSION_TABLE[usize::from(index)].as_str()
}

/// Reads a column of table indices back out of its low and high byte arrays.
pub(crate) fn version_indices(
    low: &'static Blob,
    high: &'static Blob,
) -> impl Iterator<Item = u16> {
    low.get()
        .iter()
        .zip(high.get())
        .map(|(low, high)| u16::from_le_bytes([*low, *high]))
}

pub const ANDROID_EVERGREEN_FIRST: f32 = 37.0;
pub const OP_MOB_BLINK_FIRST: u32 = 14;

#[derive(Clone, Debug)]
pub struct BrowserStat(u32, u32);

#[derive(Clone, Debug)]
pub struct VersionDetail {
    version: PooledStr,
    pub release_date: i64,
    // Use bool instead of Option to use pad space
    pub released: bool,
    pub global_usage: f32,
}

include!("generated/version-table.rs");
include!("generated/caniuse-browsers.rs");
include!("generated/caniuse-global-usage.rs");

static VERSION_LIST: LazyLock<Vec<VersionDetail>> = LazyLock::new(|| {
    version_indices(&VERSION_LIST_VERSION_LO, &VERSION_LIST_VERSION_HI)
        .zip(undelta(VERSION_LIST_RELEASE_DATE_DELTA))
        .zip(VERSION_LIST_RELEASED)
        .zip(VERSION_LIST_GLOBAL_USAGE)
        .map(
            |(((version, release_date), released), usage)| VersionDetail {
                version: VERSION_TABLE[usize::from(version)],
                release_date: i64::from(release_date),
                released: *released,
                global_usage: per_mille(*usage),
            },
        )
        .collect()
});

static BROWSERS_STATS: LazyLock<Vec<(PooledStr, BrowserStat)>> = LazyLock::new(|| {
    let mut start = 0;
    BROWSERS_STATS_KEY
        .iter()
        .zip(BROWSERS_STATS_WIDTH)
        .map(|(key, width)| {
            let end = start + u32::from(*width);
            let stat = BrowserStat(start, end);
            start = end;
            (*key, stat)
        })
        .collect()
});

static CANIUSE_BROWSERS: LazyLock<BinMap<'static, PooledStr, BrowserStat>> =
    LazyLock::new(|| BinMap(&BROWSERS_STATS));

static CANIUSE_GLOBAL_USAGE: LazyLock<Vec<(&'static str, &'static str, f32)>> =
    LazyLock::new(|| {
        CANIUSE_GLOBAL_USAGE_BROWSER
            .get()
            .iter()
            .zip(version_indices(
                &CANIUSE_GLOBAL_USAGE_VERSION_LO,
                &CANIUSE_GLOBAL_USAGE_VERSION_HI,
            ))
            .zip(CANIUSE_GLOBAL_USAGE_USAGE)
            .map(|((browser, version), usage)| {
                (
                    crate::decode_browser_name(*browser),
                    version_in_table(version),
                    per_mille(*usage),
                )
            })
            .collect()
    });

static BROWSER_VERSION_ALIASES: LazyLock<
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

pub fn iter_global_usage() -> impl ExactSizeIterator<Item = (&'static str, &'static str, f32)> {
    CANIUSE_GLOBAL_USAGE.iter().copied()
}

pub fn get_browser_version_alias(name: &str, version: &str) -> Option<&'static str> {
    BROWSER_VERSION_ALIASES.get(name)?.get(version).copied()
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

pub fn to_desktop_name(name: &str) -> Option<&'static str> {
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

pub fn normalize_version<'a>(
    name: &'a str,
    version_list: &'static [VersionDetail],
    version: &'a str,
) -> Option<&'a str> {
    if version_list.iter().any(|v| v.version.as_str() == version) {
        Some(version)
    } else if let Some(version) = get_browser_version_alias(name, version) {
        Some(version)
    } else if version_list.len() == 1 {
        version_list.first().map(|s| s.version.as_str())
    } else {
        None
    }
}

/// Looks up one browser by name, for the feature tables to resolve versions through.
pub(crate) fn browser_stat(name: &str) -> Option<&'static BrowserStat> {
    CANIUSE_BROWSERS.get(name)
}

impl BrowserStat {
    pub fn version_list(&self) -> &'static [VersionDetail] {
        &VERSION_LIST[self.range()]
    }

    /// The version list permuted into lexicographic order, so that a feature lookup can
    /// binary search it by version string.
    pub(crate) fn lex_order(&self) -> &'static [u8] {
        &VERSION_LIST_LEX_ORDER.get()[self.range()]
    }

    fn range(&self) -> std::ops::Range<usize> {
        (self.0 as usize)..(self.1 as usize)
    }
}

impl VersionDetail {
    pub fn version(&self) -> &'static str {
        self.version.as_str()
    }
}
