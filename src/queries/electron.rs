use super::Selector;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

pub(super) struct ElectronVersions(pub(super) Vec<(f32, String)>);

impl ElectronVersions {
    pub(super) fn new() -> Self {
        let versions = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/electron-to-chromium.json"
        )))
        .unwrap();

        Self(versions)
    }
}

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s+(\d+\.\d+)(?:\.\d+)?\s*-\s*(\d+\.\d+)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronSelector {
    versions: ElectronVersions,
}

impl ElectronSelector {
    pub(super) fn new() -> Self {
        Self {
            versions: ElectronVersions::new(),
        }
    }
}

impl Selector for ElectronSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        REGEX
            .captures(text)
            .and_then(|cap| {
                Some((
                    cap.get(1)?.as_str().parse::<f32>().ok()?,
                    cap.get(2)?.as_str().parse::<f32>().ok()?,
                ))
            })
            .map(|(from, to)| {
                self.versions
                    .0
                    .iter()
                    .filter(|(version, _)| from <= *version && *version <= to)
                    .map(|(_, version)| format!("chrome {}", version))
                    .collect()
            })
    }
}
