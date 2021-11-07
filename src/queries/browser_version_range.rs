use super::Selector;
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_VERSION_ALIASES},
    error::Error,
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+)\s*(>=?|<=?)\s*([\d.]+)$").unwrap());

pub(super) struct BrowserVersionRangeSelector;

impl Selector for BrowserVersionRangeSelector {
    fn select(&self, text: &str, opts: &Opts) -> Result<Option<Vec<String>>, Error> {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let name = &cap[1];
        let sign = &cap[2];
        let version = &cap[3];

        let stat = get_browser_stat(&cap[1], opts.mobile_to_desktop)
            .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
        let version: f32 = match CANIUSE_LITE_VERSION_ALIASES
            .get(&stat.name)
            .and_then(|alias| alias.get(version))
        {
            Some(version) => version.parse().unwrap_or(0.0),
            None => version.parse().unwrap_or(0.0),
        };

        let versions = stat
            .released
            .iter()
            .map(|v| v.parse::<f32>().unwrap_or(0.0))
            .filter(|v| match sign {
                ">" => *v > version,
                "<" => *v < version,
                "<=" => *v <= version,
                _ => *v >= version,
            })
            .map(|version| format!("{} {}", &stat.name, version))
            .collect();
        Ok(Some(versions))
    }
}
