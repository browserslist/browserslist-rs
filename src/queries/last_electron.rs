use super::{electron::ElectronVersions, Selector};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastElectronSelector {
    versions: ElectronVersions,
}

impl LastElectronSelector {
    pub(super) fn new() -> Self {
        Self {
            versions: ElectronVersions::new(),
        }
    }
}

impl Selector for LastElectronSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|ver| ver.as_str().parse::<usize>().ok())
            .map(|count| {
                self.versions
                    .0
                    .iter()
                    .rev()
                    .take(count)
                    .map(|(_, version)| format!("chrome {}", version))
                    .collect()
            })
    }
}
