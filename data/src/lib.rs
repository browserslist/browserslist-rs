pub mod baseline;
pub mod caniuse;
pub mod electron;
pub mod node;
mod utils;

include!("generated/firefox-esr.rs");

#[cfg(feature = "deflate")]
pub(crate) fn inflate(blob: &[u8]) -> Vec<u8> {
    miniz_oxide::inflate::decompress_to_vec(blob).expect("failed to inflate bundled data")
}

pub(crate) fn decode_release_dates(years: &[u8], months: &[u8], days: &[u8]) -> Vec<i64> {
    assert!(
        [months, days]
            .iter()
            .all(|column| column.len() == years.len()),
        "mismatched release-date column lengths"
    );
    years
        .iter()
        .zip(months)
        .zip(days)
        .map(|((&year, &month), &day)| {
            if year == 0 {
                0
            } else {
                chrono::NaiveDate::from_ymd_opt(
                    i32::from(year) + 1970,
                    u32::from(month),
                    u32::from(day),
                )
                .expect("invalid bundled release date")
                .and_hms_opt(0, 0, 0)
                .expect("invalid bundled release time")
                .and_utc()
                .timestamp()
            }
        })
        .collect()
}

const BROWSER_NAMES: [&str; 19] = [
    "ie", "edge", "firefox", "chrome", "safari", "opera", "ios_saf", "op_mini", "android", "bb",
    "op_mob", "and_chr", "and_ff", "ie_mob", "and_uc", "samsung", "and_qq", "baidu", "kaios",
];

#[doc(hidden)]
pub fn decode_browser_name(id: u8) -> &'static str {
    BROWSER_NAMES[browser_slot(id)]
}

/// Zero-based slot of a browser id within the per-browser tables.
pub(crate) fn browser_slot(id: u8) -> usize {
    usize::from(id.checked_sub(1).expect("cannot recognize browser id"))
}
