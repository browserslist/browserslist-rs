use super::{count_android_filter, should_filter_android, Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_BROWSERS},
    opts::Opts,
};

pub(super) fn last_n_browsers(count: usize, opts: &Opts) -> QueryResult {
    let distribs = CANIUSE_BROWSERS
        .keys()
        .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
        .flat_map(|(name, stat)| {
            let count = if should_filter_android(name, opts.mobile_to_desktop) {
                count_android_filter(count, opts.mobile_to_desktop)
            } else {
                count
            };

            stat.version_list
                .iter()
                .filter(|version| version.release_date.is_some())
                .rev()
                .take(count)
                .map(move |version| Distrib::new(name, &*version.version))
        })
        .collect();

    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 versions"; "basic")]
    #[test_case("last 31 versions"; "android")]
    #[test_case("last 1 version"; "support pluralization")]
    #[test_case("Last 02 Versions"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
