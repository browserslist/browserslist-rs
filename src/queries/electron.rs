use std::{collections::HashMap, fs};

pub(super) struct ElectronVersions(pub(super) Vec<(f32, String)>);

impl ElectronVersions {
    pub(super) fn new() -> Self {
        let raw = fs::read_to_string("./node_modules/electron-to-chromium/versions.js").unwrap();
        let mut versions = serde_json::from_str::<HashMap<String, String>>(
            raw.trim_start_matches("module.exports =")
                .trim_end_matches(';'),
        )
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.parse().unwrap(), v))
        .collect::<Vec<(f32, String)>>();

        versions.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

        Self(versions)
    }
}
