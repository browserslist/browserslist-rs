use crate::{data::caniuse, data::caniuse::get_browser_stat, error::Error, opts::Opts};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display};

mod browser_accurate;
mod browser_bounded_range;
mod browser_unbounded_range;
mod browserslist_config;
mod cover;
mod current_node;
mod dead;
mod defaults;
mod electron_accurate;
mod electron_bounded_range;
mod electron_unbounded_range;
mod firefox_esr;
mod last_n_browsers;
mod last_n_electron;
mod last_n_electron_major;
mod last_n_major_browsers;
mod last_n_x_browsers;
mod last_n_x_major_browsers;
mod maintained_node;
mod node_accurate;
mod node_bounded_range;
mod node_unbounded_range;
mod op_mini;
mod percentage;
mod phantom;
mod since;
mod supports;
mod unreleased_browsers;
mod unreleased_electron;
mod unreleased_x_browsers;
mod years;

/// Representation of browser name (or `node`) and its version.
///
/// When converting it to string, it will be formatted as the output of
/// [browserslist](https://github.com/browserslist/browserslist). For example:
///
/// ```
/// use browserslist::{Opts, resolve};
///
/// let distrib = &resolve(["firefox 93"], &Opts::new()).unwrap()[0];
///
/// assert_eq!(distrib.name(), "firefox");
/// assert_eq!(distrib.version(), "93");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distrib(&'static str, Cow<'static, str>);

impl Distrib {
    #[inline]
    fn new<S: Into<Cow<'static, str>>>(name: &'static str, version: S) -> Self {
        Self(name, version.into())
    }

    /// Return browser name, or `node`.
    ///
    /// ```
    /// use browserslist::{Opts, resolve};
    ///
    /// let distrib = &resolve(["firefox 93"], &Opts::new()).unwrap()[0];
    ///
    /// assert_eq!(distrib.name(), "firefox");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        self.0
    }

    /// Return version string.
    ///
    /// ```
    /// use browserslist::{Opts, resolve};
    ///
    /// let distrib = &resolve(["firefox 93"], &Opts::new()).unwrap()[0];
    ///
    /// assert_eq!(distrib.version(), "93");
    /// ```
    #[inline]
    pub fn version(&self) -> &str {
        &self.1
    }
}

impl Display for Distrib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

pub type SelectorResult = Result<Option<Vec<Distrib>>, Error>;

trait Selector {
    fn select(&self, text: &str, opts: &Opts) -> SelectorResult;
}

pub fn query(query_string: &str, opts: &Opts) -> Result<Vec<Distrib>, Error> {
    let selectors: Vec<Box<dyn Selector>> = vec![
        Box::new(last_n_major_browsers::LastNMajorBrowsersSelector),
        Box::new(last_n_browsers::LastNBrowsersSelector),
        Box::new(last_n_electron_major::LastNElectronMajorSelector),
        Box::new(last_n_x_major_browsers::LastNXMajorBrowsersSelector),
        Box::new(last_n_electron::LastNElectronSelector),
        Box::new(last_n_x_browsers::LastNXBrowsersSelector),
        Box::new(unreleased_browsers::UnreleasedBrowsersSelector),
        Box::new(unreleased_electron::UnreleasedElectronSelector),
        Box::new(unreleased_x_browsers::UnreleasedXBrowsersSelector),
        Box::new(years::YearsSelector),
        Box::new(since::SinceSelector),
        Box::new(percentage::PercentageSelector),
        Box::new(cover::CoverSelector),
        Box::new(supports::SupportsSelector),
        Box::new(electron_bounded_range::ElectronBoundedRangeSelector),
        Box::new(node_bounded_range::NodeBoundedRangeSelector),
        Box::new(browser_bounded_range::BrowserBoundedRangeSelector),
        Box::new(electron_unbounded_range::ElectronUnboundedRangeSelector),
        Box::new(node_unbounded_range::NodeUnboundedRangeSelector),
        Box::new(browser_unbounded_range::BrowserUnboundedRangeSelector),
        Box::new(firefox_esr::FirefoxESRSelector),
        Box::new(op_mini::OperaMiniSelector),
        Box::new(electron_accurate::ElectronAccurateSelector),
        Box::new(node_accurate::NodeAccurateSelector),
        Box::new(current_node::CurrentNodeSelector),
        Box::new(maintained_node::MaintainedNodeSelector),
        Box::new(phantom::PhantomSelector),
        Box::new(browser_accurate::BrowserAccurateSelector),
        Box::new(browserslist_config::BrowserslistConfigSelector),
        Box::new(defaults::DefaultsSelector),
        Box::new(dead::DeadSelector),
    ];

    for selector in selectors {
        if let Some(distribs) = selector.select(query_string, opts)? {
            return Ok(distribs);
        }
    }
    if get_browser_stat(query_string, opts.mobile_to_desktop).is_some() {
        Err(Error::VersionRequired(query_string.to_string()))
    } else {
        Err(Error::UnknownQuery(query_string.to_string()))
    }
}

#[inline]
pub fn should_filter_android(name: &str, mobile_to_desktop: bool) -> bool {
    name == "android" && !mobile_to_desktop
}

pub fn count_android_filter(count: usize, mobile_to_desktop: bool) -> usize {
    let last_released = &caniuse::get_browser_stat("android", mobile_to_desktop)
        .unwrap()
        .1
        .version_list
        .iter()
        .filter(|version| version.release_date.is_some())
        .map(|version| &*version.version)
        .last()
        .unwrap()
        .parse::<f32>()
        .unwrap();
    let diff = (last_released - caniuse::ANDROID_EVERGREEN_FIRST - (count as f32)) as usize;
    if diff > 0 {
        1
    } else {
        1 - diff
    }
}
