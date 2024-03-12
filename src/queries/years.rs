use super::{Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_BROWSERS},
    error::Error,
    opts::Opts,
};
use chrono::{Duration, Utc};

const ONE_YEAR_IN_SECONDS: f64 = 365.259641 * 24.0 * 60.0 * 60.0;

pub(super) fn years(count: f64, opts: &Opts) -> QueryResult {
    let duration =
        Duration::try_seconds((count * ONE_YEAR_IN_SECONDS) as i64).ok_or(Error::YearOverflow)?;
    let time = (Utc::now() - duration).timestamp();

    let distribs = CANIUSE_BROWSERS
        .keys()
        .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
        .flat_map(|(name, stat)| {
            stat.version_list
                .iter()
                .filter(|version| matches!(version.release_date, Some(date) if date >= time))
                .map(|version| Distrib::new(name, version.version))
        })
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 years"; "basic")]
    #[test_case("last 1 year"; "one year")]
    #[test_case("last 1.4 years"; "year fraction")]
    #[test_case("Last 5 Years"; "case insensitive")]
    #[test_case("last    2     years"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new(), None);
    }
}
