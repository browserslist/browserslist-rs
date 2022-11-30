use ahash::AHashMap;
use chrono::{NaiveDate, NaiveDateTime};
use once_cell::sync::Lazy;

pub static NODE_VERSIONS: Lazy<Vec<String>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/node-versions.json"
    )))
    .unwrap()
});

pub static RELEASE_SCHEDULE: Lazy<AHashMap<String, (NaiveDateTime, NaiveDateTime)>> =
    Lazy::new(|| {
        let date_format = "%Y-%m-%d";

        serde_json::from_str::<AHashMap<String, (String, String)>>(include_str!(concat!(
            env!("OUT_DIR"),
            "/node-release-schedule.json"
        )))
        .unwrap()
        .into_iter()
        .map(|(version, (start, end))| {
            (
                version,
                (
                    NaiveDate::parse_from_str(&start, date_format)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    NaiveDate::parse_from_str(&end, date_format)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                ),
            )
        })
        .collect()
    });
