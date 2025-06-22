use super::{count_filter_versions, Distrib, QueryResult};
use crate::{data::caniuse, opts::Opts};

pub(super) fn last_n_browsers(count: usize, opts: &Opts) -> QueryResult {
    let distribs = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            let count = count_filter_versions(name, opts.mobile_to_desktop, count);

            version_list
                .iter()
                .filter(|version| version.released)
                .rev()
                .take(count)
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

    #[test_case("last 2 versions"; "basic")]
    #[test_case("last 31 versions"; "android")]
    #[test_case("last 1 version"; "support pluralization")]
    #[test_case("Last 02 Versions"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
