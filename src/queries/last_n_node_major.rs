use super::{Distrib, QueryResult};
use crate::semver::Version;
use browserslist_data::node;
use itertools::Itertools;

pub(super) fn last_n_node_major(count: usize) -> QueryResult {
    let minimum = node::versions()
        .iter()
        .rev()
        .map(|version| {
            version
                .parse::<Version>()
                .map(|version| version.major())
                .unwrap_or_default()
        })
        .dedup()
        .nth(count - 1)
        .unwrap_or_default();

    let distribs = node::versions()
        .iter()
        .filter(|version| {
            version
                .parse::<Version>()
                .map(|version| version.major() >= minimum)
                .unwrap_or_default()
        })
        .rev()
        .map(|version| Distrib::new("node", *version))
        .collect();

    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("last 2 node major versions"; "basic")]
    #[test_case("last 2 Node major versions"; "case insensitive")]
    #[test_case("last 2 node major version"; "support pluralization")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
