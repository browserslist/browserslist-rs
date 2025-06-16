use super::{Distrib, QueryResult};
use crate::{data::caniuse, error::Error, opts::Opts};
use chrono::{LocalResult, TimeZone, Utc};

pub(super) fn since(year: i32, month: u32, day: u32, opts: &Opts) -> QueryResult {
    let time = match Utc.with_ymd_and_hms(year, month, day, 0, 0, 0) {
        LocalResult::Single(date) => date.timestamp(),
        _ => return Err(Error::InvalidDate(format!("{}-{}-{}", year, month, day))),
    };

    let distribs = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            version_list
                .iter()
                .filter(|version| version.released && version.release_date >= time)
                .map(move |version| Distrib::new(name, version.version.as_str()))
        })
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
        run_compare(query, &Opts::default(), None);
    }
}
