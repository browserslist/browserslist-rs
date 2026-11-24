use super::{Distrib, QueryResult};
use crate::{error::Error, opts::Opts};
use browserslist_data::caniuse;

pub(super) fn since(year: i32, month: u32, day: u32, opts: &Opts) -> QueryResult {
    let date = u8::try_from(month)
        .ok()
        .and_then(|month| time::Month::try_from(month).ok())
        .and_then(|month| {
            u8::try_from(day)
                .ok()
                .and_then(|day| time::Date::from_calendar_date(year, month, day).ok())
        })
        .ok_or_else(|| Error::InvalidDate(format!("{year}-{month}-{day}")))?;
    let time = date.midnight().assume_utc().unix_timestamp();

    let distribs = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            version_list
                .iter()
                .filter(|version| version.released && version.release_date >= time)
                .map(move |version| Distrib::new(name, version.version()))
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
