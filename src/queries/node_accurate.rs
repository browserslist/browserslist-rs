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
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let version = &cap[1];

            let versions = NODE_VERSIONS
                .iter()
                .filter(|v| v.split('.').zip(version.split('.')).all(|(a, b)| a == b))
                .rev()
                .next()
                .map(|version| vec![Distrib::new("node", &version)]);
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
