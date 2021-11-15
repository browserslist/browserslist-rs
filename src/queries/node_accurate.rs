use super::{Distrib, Selector, SelectorResult};
use crate::{data::node::NODE_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^node\s+(\d+(?:\.\d+(?:\.\d+)?)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct NodeAccurateSelector;

impl Selector for NodeAccurateSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let version = &cap[1];

            let versions = NODE_VERSIONS
                .iter()
                .rev()
                .find(|v| v.split('.').zip(version.split('.')).all(|(a, b)| a == b))
                .map(|version| vec![Distrib::new("node", version)]);
            if opts.ignore_unknown_versions {
                Ok(Some(versions.unwrap_or_default()))
            } else {
                versions
                    .map(Some)
                    .ok_or_else(|| Error::UnknownNodejsVersion(version.to_string()))
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{run_compare, should_failed};
    use test_case::test_case;

    #[test_case("node 7.5.0"; "basic")]
    #[test_case("Node 7.5.0"; "case insensitive")]
    #[test_case("node 5.1"; "without semver patch")]
    #[test_case("node 5"; "semver major only")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case(
        "node 3", Error::UnknownNodejsVersion(String::from("3"));
        "unknown version"
    )]
    #[test_case(
        "node 8.a", Error::UnknownQuery(String::from("node 8.a"));
        "malformed version 1"
    )]
    #[test_case(
        "node 8.8.8.8", Error::BrowserNotFound(String::from("node"));
        "malformed version 2"
    )]
    #[test_case(
        "node 8.01", Error::UnknownNodejsVersion(String::from("8.01"));
        "malformed version 3"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }

    #[test]
    fn ignore_unknown_versions() {
        run_compare("node 3", Opts::new().ignore_unknown_versions(true));
    }
}
