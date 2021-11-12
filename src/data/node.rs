use chrono::{NaiveDate, NaiveDateTime};
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static NODE_VERSIONS: Lazy<Vec<String>> =
    Lazy::new(|| serde_json::from_str(include_str!("../../data/node-versions.json")).unwrap());

pub static RELEASE_SCHEDULE: Lazy<HashMap<String, (NaiveDateTime, NaiveDateTime)>> =
    Lazy::new(|| {
        let date_format = "%Y-%m-%d";

        serde_json::from_str::<HashMap<String, (String, String)>>(include_str!(
            "../../data/node-release-schedule.json"
        ))
        .unwrap()
        .into_iter()
        .map(|(version, (start, end))| {
            (
                version,
                (
                    NaiveDate::parse_from_str(&start, date_format)
                        .unwrap()
                        .and_hms(0, 0, 0),
                    NaiveDate::parse_from_str(&end, date_format)
                        .unwrap()
                        .and_hms(0, 0, 0),
                ),
            )
        })
        .collect()
    });
