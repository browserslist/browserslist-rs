use time::Date;

include!("generated/node-versions.rs");
include!("generated/node-release-schedule.rs");

pub fn versions() -> &'static [&'static str] {
    NODE_VERSIONS
}

pub fn release_schedule(now: Date) -> impl Iterator<Item = &'static str> {
    let now = now.to_julian_day();
    let end = NODE_RELEASE_END.partition_point(|end| end <= &now);
    NODE_RELEASE_START
        .iter()
        .enumerate()
        .skip(end)
        .filter(move |(_, start)| *start < &now)
        .map(|(idx, _)| NODE_RELEASE_VERSIONS[idx])
}
