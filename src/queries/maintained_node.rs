use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::node::{NODE_VERSIONS, RELEASE_SCHEDULE},
    opts::Opts,
};
use chrono::Local;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^maintained\s+node\s+versions$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct MaintainedNodeSelector;

impl Selector for MaintainedNodeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            let now = Local::now().naive_local();

            let versions = RELEASE_SCHEDULE
                .iter()
                .filter(|(_, (start, end))| *start < now && now < *end)
                .filter_map(|(version, _)| {
                    NODE_VERSIONS
                        .iter()
                        .rev()
                        .find(|v| v.split('.').next().unwrap() == version)
                })
                .map(|version| Distrib::new("node", version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
