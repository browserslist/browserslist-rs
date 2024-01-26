use super::{Distrib, QueryResult};
use crate::{data::node::NODE_VERSIONS, semver::loose_compare};
use std::cmp::Ordering;

pub(super) fn node_bounded_range(from: &str, to: &str) -> QueryResult {
    let distribs = NODE_VERSIONS
        .iter()
        .filter(|version| {
            matches!(
                loose_compare(version, from),
                Ordering::Greater | Ordering::Equal
            ) && matches!(loose_compare(version, to), Ordering::Less | Ordering::Equal)
        })
        .map(|version| Distrib::new("node", *version))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("node 4-6"; "semver major only")]
    #[test_case("node 4-6.0.0"; "different semver formats")]
    #[test_case("node 6.5-7.5"; "with semver minor")]
    #[test_case("node 6.6.4-7.7.5"; "with semver patch")]
    #[test_case("Node 4   -    6"; "more spaces 1")]
    #[test_case("node 6.5    -  7.5"; "more spaces 2")]
    #[test_case("node 6.6.4    -    7.7.5"; "more spaces 3")]
    #[test_case("node 8.8.8.8-9.9.9.9"; "malformed version")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "node 6-8.a", Error::Nom(String::from("a"));
        "malformed version"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
