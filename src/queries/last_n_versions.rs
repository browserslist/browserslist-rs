use super::{caniuse::CANIUSE_LITE_BROWSERS, Selector};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNVersionsSelector;

impl Selector for LastNVersionsSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|ver| ver.as_str().parse::<usize>().ok())
            .map(|count| {
                CANIUSE_LITE_BROWSERS
                    .iter()
                    .map(|(name, stat)| {
                        // TODO: handle for Android
                        stat.released.iter().rev().take(count).map(|version| {
                            let mut r = name.clone();
                            r.push(' ');
                            r.push_str(&version);
                            r
                        })
                    })
                    .flatten()
                    .collect()
            })
    }
}
