use super::{
    caniuse::CANIUSE_LITE_BROWSERS, count_android_filter, should_filter_android, Selector,
};
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
        let count = REGEX
            .captures(text)?
            .get(1)?
            .as_str()
            .parse::<usize>()
            .ok()?;

        let versions = CANIUSE_LITE_BROWSERS
            .iter()
            .map(|(name, stat)| {
                // TODO: handle for Android if `mobile_to_desktop` enabled
                let count = if should_filter_android(name, false) {
                    count_android_filter(count)
                } else {
                    count
                };

                stat.released.iter().rev().take(count).map(|version| {
                    let mut r = name.clone();
                    r.push(' ');
                    r.push_str(&version);
                    r
                })
            })
            .flatten()
            .collect();

        Some(versions)
    }
}
