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
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let from = &cap[1];
            let to = &cap[2];

            let versions = NODE_VERSIONS
                .iter()
                .filter(|version| {
                    matches!(
                        loose_compare(&version, from),
                        Ordering::Greater | Ordering::Equal
                    ) && matches!(
                        loose_compare(&version, to),
                        Ordering::Less | Ordering::Equal
                    )
                })
                .map(|version| Distrib::new("node", &version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
