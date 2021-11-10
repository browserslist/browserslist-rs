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
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let sign = &cap[1];
            let version = &cap[2];

            let versions = NODE_VERSIONS
                .iter()
                .filter(|v| {
                    let ord = compare(&v, version);
                    match sign {
                        ">" => matches!(ord, Ordering::Greater),
                        "<" => matches!(ord, Ordering::Less),
                        "<=" => matches!(ord, Ordering::Less | Ordering::Equal),
                        _ => matches!(ord, Ordering::Greater | Ordering::Equal),
                    }
                })
                .map(|version| Distrib::new("node", &version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
