//! Baseline browser data, generated from the [`baseline-browser-mapping`]
//! npm package's Baseline support timeline.
//!
//! [`baseline-browser-mapping`]: https://www.npmjs.com/package/baseline-browser-mapping

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
    BASELINE_BROWSERS.iter().copied()
}

/// Minimum compatible versions (by caniuse browser name) for the Baseline
/// feature set as of `cutoff_date`, a `YYYY-MM-DD` baseline-low threshold
/// (the query date minus 30 months for widely-available queries).
///
/// Returns `None` if the date predates the first Baseline feature; every
/// version of every browser is considered compatible in that case.
pub fn min_versions_on(
    cutoff_date: &str,
) -> Option<impl Iterator<Item = (&'static str, &'static str)>> {
    let index = BASELINE_TIMELINE.partition_point(|(date, _)| *date <= cutoff_date);
    index
        .checked_sub(1)
        .map(|index| BASELINE_TIMELINE[index].1.iter().copied())
}
