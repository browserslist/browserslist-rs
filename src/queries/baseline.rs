use super::{browser_unbounded_range::browser_unbounded_range, QueryResult};
use crate::{
    opts::Opts,
    parser::{BaselineKind, Comparator},
};
use browserslist_data::baseline;
use chrono::{Datelike, Local};

type Date = (i32, u32, u32);

pub(super) fn baseline(
    kind: &BaselineKind,
    downstream: bool,
    kaios: bool,
    opts: &Opts,
) -> QueryResult {
    // The Baseline timeline is keyed by baseline-low dates, which browserslist
    // queries with a 30-month "widely available" offset:
    //   widely available          => today - 30 months
    //   newly available           => (today + 30 months) - 30 months
    //   widely available on DATE  => DATE - 30 months
    //   YEAR                      => end of that year
    let cutoff = match kind {
        BaselineKind::WidelyAvailable => add_months(today(), -30),
        BaselineKind::NewlyAvailable => add_months(add_months(today(), 30), -30),
        BaselineKind::WidelyAvailableOnDate(date) => add_months(parse_date(date), -30),
        BaselineKind::Year(year) => (i32::from(*year), 12, 31),
    };
    let cutoff = date_key(cutoff);

    // KaiOS is only included when downstream browsers are requested as well.
    let is_included = |browser: &str| {
        baseline::is_core_browser(browser) || (downstream && (kaios || browser != "kaios"))
    };

    let mut distribs = Vec::new();
    match baseline::min_versions_on(cutoff) {
        Some(min_versions) => {
            for (browser, version) in min_versions.filter(|(browser, _)| is_included(browser)) {
                distribs.append(&mut browser_unbounded_range(
                    browser,
                    Comparator::GreaterOrEqual,
                    version,
                    opts,
                )?);
            }
        }
        // Before the first Baseline feature, every version of every browser
        // is considered compatible.
        None => {
            for browser in baseline::browsers().filter(|browser| is_included(browser)) {
                distribs.append(&mut browser_unbounded_range(
                    browser,
                    Comparator::GreaterOrEqual,
                    "0",
                    opts,
                )?);
            }
        }
    }
    Ok(distribs)
}

/// Encodes a date as decimal `yyyymmdd`, the key of the Baseline timeline.
fn date_key((year, month, day): Date) -> u32 {
    year as u32 * 10000 + month * 100 + day
}

fn today() -> Date {
    let now = Local::now().date_naive();
    (now.year(), now.month(), now.day())
}

fn parse_date(date: &str) -> Date {
    // The "YYYY-MM-DD" shape is guaranteed by the parser.
    let mut parts = date.split('-').map(|part| part.parse().unwrap());
    let year = parts.next().unwrap();
    let month = parts.next().unwrap() as u32;
    let day = parts.next().unwrap() as u32;
    (year, month, day)
}

/// Adds `offset` months to a date with JavaScript `Date.prototype.setMonth`
/// semantics: a day-of-month past the end of the target month rolls over
/// into the following month.
fn add_months((year, month, day): Date, offset: i32) -> Date {
    let months = year * 12 + month as i32 - 1 + offset;
    let mut year = months.div_euclid(12);
    let mut month = months.rem_euclid(12) as u32 + 1;
    let mut day = day;
    while day > days_in_month(year, month) {
        day -= days_in_month(year, month);
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    (year, month, day)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("baseline widely available"; "widely")]
    #[test_case("BASELINE WIDELY AVAILABLE"; "case insensitive widely")]
    #[test_case("baseline newly available"; "newly")]
    #[test_case("baseline 2014"; "year before baseline")]
    #[test_case("baseline 2015"; "year 2015")]
    #[test_case("baseline 2016"; "year 2016")]
    #[test_case("baseline 2017"; "year 2017")]
    #[test_case("baseline 2018"; "year 2018")]
    #[test_case("baseline 2019"; "year 2019")]
    #[test_case("baseline 2020"; "year 2020")]
    #[test_case("baseline 2021"; "year 2021")]
    #[test_case("baseline 2022"; "year 2022")]
    #[test_case("baseline 2023"; "year 2023")]
    #[test_case("baseline 2024"; "year 2024")]
    #[test_case("baseline 2025"; "year 2025")]
    #[test_case("baseline widely available on 2018-01-01"; "widely on date before baseline")]
    #[test_case("baseline widely available on 2021-01-01"; "widely on date 2021-01-01")]
    #[test_case("baseline widely available on 2023-04-05"; "widely on date 2023-04-05")]
    #[test_case("baseline widely available on 2024-05-31"; "widely on date with day overflow")]
    #[test_case("baseline widely available on 2024-06-15"; "widely on date 2024-06-15")]
    #[test_case("baseline 2022 with downstream"; "year with downstream")]
    #[test_case("baseline widely available with downstream"; "widely with downstream")]
    #[test_case("baseline newly available with downstream"; "newly with downstream")]
    #[test_case("baseline 2020 including kaios"; "kaios without downstream")]
    #[test_case("baseline 2020 with downstream including kaios"; "year with downstream and kaios")]
    #[test_case(
        "baseline widely available on 2024-06-15 with downstream including kaios";
        "widely on date with downstream and kaios"
    )]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test_case("baseline widely available"; "widely")]
    #[test_case("baseline 2022 with downstream"; "year with downstream")]
    fn mobile_to_desktop(query: &str) {
        run_compare(
            query,
            &Opts {
                mobile_to_desktop: true,
                ..Default::default()
            },
            None,
        );
    }
}
