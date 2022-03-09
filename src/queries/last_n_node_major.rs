use super::{Distrib, QueryResult};
use crate::{data::node::NODE_VERSIONS, semver::Version};
use itertools::Itertools;

pub(super) fn last_n_node_major(count: usize) -> QueryResult {
    let minimum = NODE_VERSIONS
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

    let distribs = NODE_VERSIONS
        .iter()
        .filter(|version| {
            version
                .parse::<Version>()
                .map(|version| version.major() >= minimum)
                .unwrap_or_default()
        })
        .rev()
        .map(|version| Distrib::new("node", version))
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
        run_compare(query, &Opts::new());
    }
}
