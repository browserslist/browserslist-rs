use super::{Distrib, Selector, SelectorResult};
use crate::{
    data::caniuse::{get_browser_stat, normalize_version},
    error::Error,
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(\w+)\s+(tp|[\d.]+)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct BrowserAccurateSelector;

impl Selector for BrowserAccurateSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let name = &cap[1];
            let version = match &cap[2] {
                version if version.eq_ignore_ascii_case("tp") => "TP",
                version => version,
            };

            let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
                .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;

            if let Some(version) = normalize_version(stat, version) {
                Ok(Some(vec![Distrib::new(name, version.to_owned())]))
            } else {
                let version = if version.contains('.') {
                    Cow::Borrowed(version.trim_end_matches(".0"))
                } else {
                    let mut v = version.to_owned();
                    v.push_str(".0");
                    Cow::Owned(v)
                };
                if let Some(version) = normalize_version(stat, &version) {
                    Ok(Some(vec![Distrib::new(name, version.to_owned())]))
                } else if opts.ignore_unknown_versions {
                    Ok(Some(vec![]))
                } else {
                    Err(Error::UnknownBrowserVersion(
                        cap[1].to_string(),
                        version.to_string(),
                    ))
                }
            }
        } else {
            Ok(None)
        }
    }
}
