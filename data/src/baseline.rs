//! Baseline browser data, generated from the [`baseline-browser-mapping`]
//! npm package's Baseline support timeline.
//!
//! [`baseline-browser-mapping`]: https://www.npmjs.com/package/baseline-browser-mapping

use crate::{blob::Blob, caniuse::version_in_table, decode_browser_name, utils::undelta};
use std::sync::LazyLock;

include!("generated/baseline.rs");

static BASELINE_VERSIONS: LazyLock<Vec<(u8, &'static str)>> = LazyLock::new(|| {
    BASELINE_VERSIONS_BROWSER
        .get()
        .iter()
        .zip(crate::caniuse::version_indices(
            &BASELINE_VERSIONS_VERSION_LO,
            &BASELINE_VERSIONS_VERSION_HI,
        ))
        .map(|(browser, version)| (*browser, version_in_table(version)))
        .collect()
});

static BASELINE_TIMELINE: LazyLock<Vec<(u32, u16, u16)>> = LazyLock::new(|| {
    let mut start = 0;
    undelta(BASELINE_TIMELINE_DATE_DELTA)
        .zip(BASELINE_TIMELINE_WIDTH)
        .map(|(date, width)| {
            let end = start + u16::from(*width);
            let entry = (date, start, end);
            start = end;
            entry
        })
        .collect()
});

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
    let index = BASELINE_TIMELINE.partition_point(|(date, ..)| *date <= cutoff_date);
    index.checked_sub(1).map(|index| {
        let (_, start, end) = BASELINE_TIMELINE[index];
        BASELINE_VERSIONS[usize::from(start)..usize::from(end)]
            .iter()
            .map(|(id, version)| (decode_browser_name(*id), *version))
    })
}
