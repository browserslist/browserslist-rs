use super::{Distrib, QueryResult};
use crate::{data::caniuse, opts::Opts};

pub(super) fn unreleased_browsers(opts: &Opts) -> QueryResult {
    let distribs = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            version_list
                .iter()
                .filter(|version| !version.released)
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

    #[test_case("unreleased versions"; "basic")]
    #[test_case("Unreleased Versions"; "case insensitive")]
    #[test_case("unreleased        versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
