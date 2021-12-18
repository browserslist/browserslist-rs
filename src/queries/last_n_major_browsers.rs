use super::{count_android_filter, should_filter_android, Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_BROWSERS},
    opts::Opts,
};
use itertools::Itertools;

pub(super) fn last_n_major_browsers(count: usize, opts: &Opts) -> QueryResult {
    let distribs = CANIUSE_BROWSERS
        .keys()
        .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
        .map(|(name, stat)| {
            let count = if should_filter_android(name, opts.mobile_to_desktop) {
                count_android_filter(count, opts.mobile_to_desktop)
            } else {
                count
            };

            let minimum: u32 = stat
                .version_list
                .iter()
                .filter(|version| version.release_date.is_some())
                .rev()
                .map(|version| version.version.split('.').next().unwrap())
                .dedup()
                .nth(count - 1)
                .and_then(|minimum| minimum.parse().ok())
                .unwrap_or(0);

            stat.version_list
                .iter()
                .filter(|version| version.release_date.is_some())
                .map(|version| &*version.version)
                .filter(move |version| {
                    version.split('.').next().unwrap().parse().unwrap_or(0) >= minimum
                })
                .rev()
                .map(move |version| Distrib::new(name, version))
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

    #[test_case("last 2 major versions"; "basic")]
    #[test_case("last 1 major version"; "support pluralization")]
    #[test_case("Last 01 MaJoR Version"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
