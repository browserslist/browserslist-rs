//! Baseline browser data, generated from the [`baseline-browser-mapping`]
//! npm package's Baseline support timeline.
//!
//! [`baseline-browser-mapping`]: https://www.npmjs.com/package/baseline-browser-mapping

use crate::{decode_browser_name, utils::PooledStr};
#[cfg(feature = "deflate")]
use std::sync::LazyLock;

include!("generated/baseline.rs");

/// caniuse names of the seven core Baseline browsers; the remaining browsers
/// in the dataset are downstream browsers sharing a core browser's engine.
static CORE_BROWSERS: &[&str] = &[
    "and_chr", "and_ff", "chrome", "edge", "firefox", "ios_saf", "safari",
];

pub fn is_core_browser(name: &str) -> bool {
    CORE_BROWSERS.contains(&name)
}

/// All browsers tracked by the Baseline dataset, by caniuse name.
pub fn browsers() -> impl Iterator<Item = &'static str> {
    BASELINE_BROWSERS.iter().map(|&id| decode_browser_name(id))
}

/// Minimum compatible versions (by caniuse browser name) for the Baseline
/// feature set as of `cutoff_date`, a baseline-low threshold encoded as
/// decimal `yyyymmdd` (the query date minus 30 months for widely-available
/// queries).
///
/// Returns `None` if the date predates the first Baseline feature; every
/// version of every browser is considered compatible in that case.
pub fn min_versions_on(
    cutoff_date: u32,
) -> Option<impl Iterator<Item = (&'static str, &'static str)>> {
    let index = BASELINE_TIMELINE_DATE.partition_point(|date| *date <= cutoff_date);
    index.checked_sub(1).map(|index| {
        let start = BASELINE_TIMELINE_START[index];
        let end = BASELINE_TIMELINE_END[index];
        BASELINE_VERSION_BROWSER[usize::from(start)..usize::from(end)]
            .iter()
            .zip(&BASELINE_VERSION_VERSION[usize::from(start)..usize::from(end)])
            .map(|(id, version)| {
                (
                    decode_browser_name(*id),
                    PooledStr(BASELINE_VERSION_TABLE[usize::from(*version)]).as_str(),
                )
            })
    })
}
