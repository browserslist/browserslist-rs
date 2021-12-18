use super::{Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_BROWSERS},
    opts::Opts,
};

pub(super) fn unreleased_browsers(opts: &Opts) -> QueryResult {
    let distribs = CANIUSE_BROWSERS
        .keys()
        .filter_map(|name| get_browser_stat(name, opts.mobile_to_desktop))
        .map(|(name, stat)| {
            stat.version_list
                .iter()
                .filter(|version| version.release_date.is_none())
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

    #[test_case("unreleased versions"; "basic")]
    #[test_case("Unreleased Versions"; "case insensitive")]
    #[test_case("unreleased        versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
