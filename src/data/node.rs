use ahash::AHashMap;
use chrono::{NaiveDate, NaiveDateTime};
use std::sync::LazyLock;

pub static NODE_VERSIONS: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| include!("../generated/node-versions.rs"));

pub static RELEASE_SCHEDULE: LazyLock<AHashMap<&'static str, (NaiveDateTime, NaiveDateTime)>> =
    LazyLock::new(|| {
        let date_format = "%Y-%m-%d";

        include!("../generated/node-release-schedule.rs")
            .into_iter()
            .map(|(version, (start, end))| {
                (
                    version,
                    (
                        NaiveDate::parse_from_str(start, date_format)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap(),
                        NaiveDate::parse_from_str(end, date_format)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap(),
                    ),
                )
            })
            .collect()
    });
