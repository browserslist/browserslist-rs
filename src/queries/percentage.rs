use super::{Distrib, Selector, SelectorResult};
use crate::{data::caniuse::CANIUSE_BROWSERS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([<>]=?)\s*(\d*\.?\d+)%$").unwrap());

pub(super) struct PercentageSelector;

impl Selector for PercentageSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };

        let sign = &cap[1];
        let popularity: f32 = cap[2].parse().map_err(Error::ParsePercentage)?;

        let versions = CANIUSE_BROWSERS
            .iter()
            .map(|(name, stat)| {
                stat.version_list
                    .iter()
                    .filter(|version| {
                        let usage = version.global_usage;
                        match sign {
                            ">" => usage > popularity,
                            "<" => usage < popularity,
                            "<=" => usage <= popularity,
                            _ => usage >= popularity,
                        }
                    })
                    .map(|version| Distrib::new(name, &*version.version))
            })
            .flatten()
            .collect();
        Ok(Some(versions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("> 10%"; "greater")]
    #[test_case(">= 5%"; "greater or equal")]
    #[test_case("< 5%"; "less")]
    #[test_case("<= 5%"; "less or equal")]
    #[test_case(">10%"; "no space")]
    #[test_case("> 10.2%"; "with float")]
    #[test_case("> .2%"; "with float that has a leading dot")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }
}
