use super::{Distrib, Selector, SelectorResult};
use crate::{data::node::NODE_VERSIONS, opts::Opts, semver::loose_compare};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::cmp::Ordering;

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^node\s+(\d+(?:\.\d+(?:\.\d+)?)?)\s*-\s*(\d+(?:\.\d+(?:\.\d+)?)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct NodeBoundedRangeSelector;

impl Selector for NodeBoundedRangeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let from = &cap[1];
            let to = &cap[2];

            let versions = NODE_VERSIONS
                .iter()
                .filter(|version| {
                    matches!(
                        loose_compare(version, from),
                        Ordering::Greater | Ordering::Equal
                    ) && matches!(loose_compare(version, to), Ordering::Less | Ordering::Equal)
                })
                .map(|version| Distrib::new("node", version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::Error,
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
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "node 6-8.a", Error::UnknownQuery(String::from("node 6-8.a"));
        "malformed version 1"
    )]
    #[test_case(
        "node 8.8.8.8-9", Error::UnknownNodejsVersion(String::from("8.8.8.8 - 9"));
        "malformed version 2"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
