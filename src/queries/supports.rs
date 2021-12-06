use super::{Distrib, Selector, SelectorResult};
use crate::{data::features::get_feature_stat, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^supports\s+([\w-]+)$").unwrap());

pub(super) struct SupportsSelector;

impl Selector for SupportsSelector {
    fn select(&self, text: &str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let name = &cap[1];
            if let Some(feature) = get_feature_stat(name) {
                let distribs = feature
                    .iter()
                    .map(|(name, version)| Distrib::new(&*name, *version))
                    .collect();
                Ok(Some(distribs))
            } else {
                Err(Error::UnknownBrowserFeature(name.to_string()))
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{run_compare, should_failed};
    use test_case::test_case;

    #[test_case("supports objectrtc"; "case 1")]
    #[test_case("supports rtcpeerconnection"; "case 2")]
    #[test_case("supports arrow-functions"; "case 3")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("supports xxxyyyzzz", &Opts::new()),
            Error::UnknownBrowserFeature(String::from("xxxyyyzzz"))
        );
    }
}
