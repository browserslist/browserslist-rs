use chrono::NaiveDate;

include!("generated/node-versions.rs");
include!("generated/node-release-schedule.rs");

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
