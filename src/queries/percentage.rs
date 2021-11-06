use super::{caniuse::CANIUSE_LITE_USAGE, Selector};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(>=?|<=?)\s*(\d+|\d+\.\d+|\.\d+)%").unwrap());

pub(super) struct PercentageSelector;

impl Selector for PercentageSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        let matches = REGEX.captures(text)?;
        let sign = matches.get(1)?.as_str();
        let popularity: f32 = matches.get(2)?.as_str().parse().ok()?;

        let versions = CANIUSE_LITE_USAGE
            .iter()
            .filter(|(_, usage)| match sign {
                ">" => **usage > popularity,
                "<" => **usage < popularity,
                "<=" => **usage <= popularity,
                _ => **usage >= popularity,
            })
            .map(|(version, _)| version.clone())
            .collect();
        Some(versions)
    }
}
