use super::{Distrib, Selector, SelectorResult};
use crate::{data::node::NODE_VERSIONS, opts::Opts, semver::compare};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::cmp::Ordering;

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^node\s*([<>]=?)\s*(\d+(?:\.\d+(?:\.\d+)?)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct NodeUnboundedRangeSelector;

impl Selector for NodeUnboundedRangeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let sign = &cap[1];
            let version = &cap[2];

            let versions = NODE_VERSIONS
                .iter()
                .filter(|v| {
                    let ord = compare(v, version);
                    match sign {
                        ">" => matches!(ord, Ordering::Greater),
                        "<" => matches!(ord, Ordering::Less),
                        "<=" => matches!(ord, Ordering::Less | Ordering::Equal),
                        _ => matches!(ord, Ordering::Greater | Ordering::Equal),
                    }
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
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("node <= 5"; "less or equal")]
    #[test_case("node < 5"; "less")]
    #[test_case("node >= 9"; "greater or equal")]
    #[test_case("node > 9"; "greater")]
    #[test_case("Node <= 5"; "case insensitive")]
    #[test_case("node > 10.12"; "with semver minor")]
    #[test_case("node > 10.12.1"; "with semver patch")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
