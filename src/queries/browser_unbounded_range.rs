use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_VERSION_ALIASES},
    error::Error,
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+)\s*([<>]=?)\s*([\d.]+)$").unwrap());

pub(super) struct BrowserUnboundedRangeSelector;

impl Selector for BrowserUnboundedRangeSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let name = &cap[1];
        let sign = &cap[2];
        let version = &cap[3];

        let (name, stat) = get_browser_stat(&cap[1], opts.mobile_to_desktop)
            .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
        let version = CANIUSE_LITE_VERSION_ALIASES
            .get(name)
            .and_then(|alias| alias.get(version).map(|s| s.as_str()))
            .unwrap_or(version)
            .parse()
            .unwrap_or(0.0);

        let versions = stat
            .released
            .iter()
            .filter(|v| {
                let v = v.parse().unwrap_or(0.0);
                match sign {
                    ">" => v > version,
                    "<" => v < version,
                    "<=" => v <= version,
                    _ => v >= version,
                }
            })
            .map(|version| Distrib::new(name, version))
            .collect();
        Ok(Some(versions))
    }
}
