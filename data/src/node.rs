use chrono::NaiveDate;
use std::sync::LazyLock;

include!("generated/node-versions.rs");
include!("generated/node-release-schedule.rs");

static NODE_RELEASE_SCHEDULE: LazyLock<Vec<(NaiveDate, NaiveDate)>> = LazyLock::new(|| {
    (0..NODE_RELEASE_START.len())
        .map(|index| (NODE_RELEASE_START[index], NODE_RELEASE_END[index]))
        .collect()
});

pub fn versions() -> &'static [&'static str] {
    NODE_VERSIONS
}

pub fn release_schedule(now: NaiveDate) -> impl Iterator<Item = &'static str> {
    let end = NODE_RELEASE_SCHEDULE.partition_point(|(_, end)| end <= &now);
    NODE_RELEASE_SCHEDULE
        .iter()
        .enumerate()
        .skip(end)
        .filter(move |(_, (start, _))| start < &now)
        .map(|(idx, _)| NODE_RELEASE_VERSIONS[idx])
}
