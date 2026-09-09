pub mod baseline;
pub mod caniuse;
pub mod electron;
pub mod node;
mod utils;

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
#[doc(hidden)]
pub fn decode_browser_name(id: u8) -> &'static str {
    match id {
        1 => "ie",
        2 => "edge",
        3 => "firefox",
        4 => "chrome",
        5 => "safari",
        6 => "opera",
        7 => "ios_saf",
        8 => "op_mini",
        9 => "android",
        10 => "bb",
        11 => "op_mob",
        12 => "and_chr",
        13 => "and_ff",
        14 => "ie_mob",
        15 => "and_uc",
        16 => "samsung",
        17 => "and_qq",
        18 => "baidu",
        19 => "kaios",
        _ => unreachable!("cannot recognize browser id"),
    }
}
