use super::{count_filter_versions, Distrib, QueryResult};
use crate::opts::Opts;
use browserslist_data::caniuse;
use itertools::Itertools;

pub(super) fn last_n_major_browsers(count: usize, opts: &Opts) -> QueryResult {
    let distribs = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            let count = count_filter_versions(name, opts.mobile_to_desktop, count);

            let minimum: u32 = version_list
                .iter()
                .filter(|version| version.released)
                .rev()
                .map(|version| version.version().split('.').next().unwrap())
                .dedup()
                .nth(count - 1)
                .and_then(|minimum| minimum.parse().ok())
                .unwrap_or(0);

            version_list
                .iter()
                .filter(|version| version.released)
                .map(|version| version.version())
                .filter(move |version| {
                    version.split('.').next().unwrap().parse().unwrap_or(0) >= minimum
                })
                .rev()
                .map(move |version| Distrib::new(name, version))
        })
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
        run_compare(query, &Opts::default(), None);
    }
}
