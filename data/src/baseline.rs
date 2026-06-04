static BASELINE_WIDELY: &[(&str, &str)] = include!("generated/baseline-widely.rs");
static BASELINE_NEWLY: &[(&str, &str)] = include!("generated/baseline-newly.rs");
static BASELINE_YEARS: &[(u16, &str, &str)] = include!("generated/baseline-years.rs");
// (cutoff_date, chrome, chrome_android, edge, firefox, firefox_android, safari, safari_ios)
// Cumulative max min-versions, sorted by cutoff_date.
static BASELINE_FEATURES: &[(&str, &str, &str, &str, &str, &str, &str, &str)] =
    include!("generated/baseline-features.rs");
include!("generated/baseline-downstream.rs");

const CORE_BROWSER_ORDER: &[&str] =
    &["chrome", "and_chr", "edge", "firefox", "and_ff", "safari", "ios_saf"];

fn get_core_min_from_snapshot(
    snapshot: (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str),
    browser: &str,
) -> Option<&'static str> {
    let (_, c, ca, e, f, fa, s, si) = snapshot;
    let v = match browser {
        "chrome" => c,
        "and_chr" => ca,
        "edge" => e,
        "firefox" => f,
        "and_ff" => fa,
        "safari" => s,
        "ios_saf" => si,
        _ => return None,
    };
    if v == "0" { None } else { Some(v) }
}

pub fn get_baseline_widely_min_version(browser: &str) -> Option<&'static str> {
    BASELINE_WIDELY
        .binary_search_by_key(&browser, |(b, _)| *b)
        .ok()
        .map(|i| BASELINE_WIDELY[i].1)
}

pub fn get_baseline_newly_min_version(browser: &str) -> Option<&'static str> {
    BASELINE_NEWLY
        .binary_search_by_key(&browser, |(b, _)| *b)
        .ok()
        .map(|i| BASELINE_NEWLY[i].1)
}

pub fn get_baseline_year_min_version(year: u16, browser: &str) -> Option<&'static str> {
    // The CSV year Y for a version V means V is in the range [min_for_Y, min_for_{Y+1}).
    // For baseline:YYYY, we want the minimum version = the first version in the smallest
    // CSV year group Y where Y >= YYYY.
    //
    // Example: Chrome has no year=2018 group; Chrome 66 (year=2019) is the correct
    // minimum for both baseline:2018 and baseline:2019.
    // Example: Safari starts at year=2017; Safari 11 is the minimum for baseline:2015/2016/2017.
    BASELINE_YEARS
        .iter()
        .filter(|(y, b, _)| *y >= year && *b == browser)
        .min_by_key(|(y, _, _)| *y)
        .map(|(_, _, v)| *v)
}

/// Returns the minimum version for a given browser as of a specific cutoff date.
/// The cutoff_date is the `baseline_low_date` threshold (already with 30-month offset applied
/// for widely-available queries).
pub fn get_baseline_cutoff_date_min_version(
    cutoff_date: &str,
    browser: &str,
) -> Option<&'static str> {
    // Binary search for the largest entry with date <= cutoff_date.
    // BASELINE_FEATURES is sorted by date strings (YYYY-MM-DD is lexicographically sortable).
    let idx = BASELINE_FEATURES
        .partition_point(|(date, ..)| *date <= cutoff_date);

    if idx == 0 {
        return None;
    }
    get_core_min_from_snapshot(BASELINE_FEATURES[idx - 1], browser)
}

/// Returns the minimum version for a Blink-based downstream browser given the minimum
/// Chrome (Blink) engine major version.
pub fn get_downstream_blink_min_version(
    chrome_min: &str,
    caniuse_browser: &str,
) -> Option<&'static str> {
    let min_major: u16 = chrome_min.split('.').next()?.parse().ok()?;
    // Find the minimum browser_version where engine_version >= min_major.
    // Entries are not guaranteed monotone in (engine_version, browser_version) order,
    // so we scan all entries for this browser.
    BASELINE_DOWNSTREAM_BLINK
        .iter()
        .filter(|(b, ev, _)| *b == caniuse_browser && *ev >= min_major)
        .min_by(|(_, _, va), (_, _, vb)| cmp_version(va, vb))
        .map(|(_, _, v)| *v)
}

/// Returns the minimum version for a Gecko-based downstream browser given the minimum
/// Firefox (Gecko) engine major version.
pub fn get_downstream_gecko_min_version(
    firefox_min: &str,
    caniuse_browser: &str,
) -> Option<&'static str> {
    let min_major: u16 = firefox_min.split('.').next()?.parse().ok()?;
    BASELINE_DOWNSTREAM_GECKO
        .iter()
        .filter(|(b, ev, _)| *b == caniuse_browser && *ev >= min_major)
        .min_by(|(_, _, va), (_, _, vb)| cmp_version(va, vb))
        .map(|(_, _, v)| *v)
}

fn cmp_version(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> (u64, u64) {
        let mut parts = s.split('.');
        let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        (major, minor)
    };
    parse(a).cmp(&parse(b))
}

/// Returns the caniuse browser names for Blink-based downstream browsers.
pub fn blink_downstream_browsers() -> impl Iterator<Item = &'static str> {
    let mut seen = std::collections::BTreeSet::new();
    BASELINE_DOWNSTREAM_BLINK
        .iter()
        .filter_map(move |(b, _, _)| {
            if seen.insert(*b) { Some(*b) } else { None }
        })
}

/// Returns the caniuse browser names for Gecko-based downstream browsers.
pub fn gecko_downstream_browsers() -> impl Iterator<Item = &'static str> {
    let mut seen = std::collections::BTreeSet::new();
    BASELINE_DOWNSTREAM_GECKO
        .iter()
        .filter_map(move |(b, _, _)| {
            if seen.insert(*b) { Some(*b) } else { None }
        })
}
