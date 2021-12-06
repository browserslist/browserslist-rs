use super::{Distrib, Selector, SelectorResult};
use crate::{data::region::get_usage_by_region, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([<>]=?)\s*(\d*\.?\d+)%\s+in\s+((?:alt-)?\w\w)$").unwrap());

pub(super) struct PercentageByRegionSelector;

impl Selector for PercentageByRegionSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let sign = &cap[1];
            let popularity: f32 = cap[2].parse().map_err(Error::ParsePercentage)?;
            let region = &cap[3];
            let region = if region.len() == 2 {
                region.to_uppercase()
            } else {
                region.to_lowercase()
            };

            if let Some(region_data) = get_usage_by_region(&region) {
                let distribs = region_data
                    .iter()
                    .filter(|(_, _, usage)| match sign {
                        ">" => *usage > popularity,
                        "<" => *usage < popularity,
                        "<=" => *usage <= popularity,
                        _ => *usage >= popularity,
                    })
                    .map(|(name, version, _)| Distrib::new(&*name, *version))
                    .collect();
                Ok(Some(distribs))
            } else {
                Err(Error::UnknownRegion(cap[3].to_string()))
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

    #[test_case("> 10% in US"; "greater")]
    #[test_case(">= 5% in US"; "greater or equal")]
    // TODO: Tests ignored.
    // In JS-implementation, `null < percentage` will be `true`,
    // however some browser usages in Can I Use are `null` which causes those browsers
    // will be included in final result.
    // #[test_case("< 5% in US"; "less")]
    // #[test_case("<= 5% in US"; "less or equal")]
    #[test_case("> 10.2% in US"; "with float")]
    #[test_case("> .2% in US"; "with float that has a leading dot")]
    #[test_case("> 10.2% in us"; "fixes country case")]
    #[test_case("> 1% in RU"; "load country")]
    #[test_case("> 1% in alt-AS"; "load continents")]
    #[test_case(">10% in US"; "no space")]
    #[test_case("> 1% in CN"; "normalize incorrect caniuse versions for and-prefixed")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("> 1% in XX", &Opts::new()),
            Error::UnknownRegion(String::from("XX"))
        );
    }
}
