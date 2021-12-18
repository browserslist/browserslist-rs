use super::{Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_BROWSERS},
    error::Error,
    opts::Opts,
};
use chrono::{LocalResult, TimeZone, Utc};

pub(super) fn since(year: i32, month: u32, day: u32, opts: &Opts) -> QueryResult {
    let time = match Utc.ymd_opt(year, month, day) {
        LocalResult::Single(date) => date.and_hms(0, 0, 0).timestamp(),
        _ => return Err(Error::InvalidDate(format!("{}-{}-{}", year, month, day))),
    };

    let distribs = CANIUSE_BROWSERS
        .keys()
        .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
        .map(|(name, stat)| {
            stat.version_list
                .iter()
                .filter(|version| matches!(version.release_date, Some(date) if date >= time))
                .map(|version| Distrib::new(name, &*version.version))
        })
        .flatten()
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("since 2017"; "year only")]
    #[test_case("Since 2017"; "case insensitive")]
    #[test_case("since 2017-02"; "with month")]
    #[test_case("since 2017-02-15"; "with day")]
    #[test_case("since 1970"; "unix timestamp zero")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
