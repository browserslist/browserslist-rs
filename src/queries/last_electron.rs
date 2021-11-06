use super::Selector;
use crate::{data::electron::ELECTRON_VERSIONS, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastElectronSelector;

impl Selector for LastElectronSelector {
    fn select(&self, text: &str, _: &Opts) -> Option<Vec<String>> {
        REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|ver| ver.as_str().parse::<usize>().ok())
            .map(|count| {
                ELECTRON_VERSIONS
                    .iter()
                    .rev()
                    .take(count)
                    .map(|(_, version)| format!("chrome {}", version))
                    .collect()
            })
    }
}
