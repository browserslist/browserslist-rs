use crate::{
    data::caniuse,
    error::Error,
    opts::Opts,
    parser::{QueryAtom, Stats, VersionRange},
};
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
mod percentage_by_region;
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

pub type QueryResult = Result<Vec<Distrib>, Error>;

pub fn query(atom: QueryAtom, opts: &Opts) -> QueryResult {
    match atom {
        QueryAtom::Last {
            count,
            major,
            name: Some(name),
        } if name.eq_ignore_ascii_case("electron") => {
            let count = count as usize;
            if major {
                last_n_electron_major::last_n_electron_major(count)
            } else {
                last_n_electron::last_n_electron(count)
            }
        }
        QueryAtom::Last {
            count,
            major,
            name: Some(name),
        } => {
            let count = count as usize;
            if major {
                last_n_x_major_browsers::last_n_x_major_browsers(count, name, opts)
            } else {
                last_n_x_browsers::last_n_x_browsers(count, name, opts)
            }
        }
        QueryAtom::Last {
            count,
            major,
            name: None,
        } => {
            let count = count as usize;
            if major {
                last_n_major_browsers::last_n_major_browsers(count, opts)
            } else {
                last_n_browsers::last_n_browsers(count, opts)
            }
        }
        QueryAtom::Unreleased(Some(name)) if name.eq_ignore_ascii_case("electron") => {
            unreleased_electron::unreleased_electron()
        }
        QueryAtom::Unreleased(Some(name)) => {
            unreleased_x_browsers::unreleased_x_browsers(name, opts)
        }
        QueryAtom::Unreleased(None) => unreleased_browsers::unreleased_browsers(opts),
        QueryAtom::Years(count) => years::years(count, opts),
        QueryAtom::Since { year, month, day } => since::since(year, month, day, opts),
        QueryAtom::Percentage {
            comparator,
            popularity,
            stats: Stats::Global,
        } => percentage::percentage(comparator, popularity),
        QueryAtom::Percentage {
            comparator,
            popularity,
            stats: Stats::Region(region),
        } => percentage_by_region::percentage_by_region(comparator, popularity, region),
        QueryAtom::Cover { coverage, .. } => cover::cover(coverage),
        QueryAtom::Supports(name) => supports::supports(name),
        QueryAtom::Electron(VersionRange::Bounded(from, to)) => {
            electron_bounded_range::electron_bounded_range(from, to)
        }
        QueryAtom::Electron(VersionRange::Unbounded(comparator, version)) => {
            electron_unbounded_range::electron_unbounded_range(comparator, version)
        }
        QueryAtom::Electron(VersionRange::Accurate(version)) => {
            electron_accurate::electron_accurate(version)
        }
        QueryAtom::Node(VersionRange::Bounded(from, to)) => {
            node_bounded_range::node_bounded_range(from, to)
        }
        QueryAtom::Node(VersionRange::Unbounded(comparator, version)) => {
            node_unbounded_range::node_unbounded_range(comparator, version)
        }
        QueryAtom::Node(VersionRange::Accurate(version)) => {
            node_accurate::node_accurate(version, opts)
        }
        QueryAtom::Browser(name, VersionRange::Bounded(from, to)) => {
            browser_bounded_range::browser_bounded_range(name, from, to, opts)
        }
        QueryAtom::Browser(name, VersionRange::Unbounded(comparator, version)) => {
            browser_unbounded_range::browser_unbounded_range(name, comparator, version, opts)
        }
        QueryAtom::Browser(name, VersionRange::Accurate(version)) => {
            browser_accurate::browser_accurate(name, version, opts)
        }
        QueryAtom::FirefoxESR => firefox_esr::firefox_esr(),
        QueryAtom::OperaMini => op_mini::op_mini(),
        QueryAtom::CurrentNode => current_node::current_node(),
        QueryAtom::MaintainedNode => maintained_node::maintained_node(),
        QueryAtom::Phantom(is_later_version) => phantom::phantom(is_later_version),
        QueryAtom::BrowserslistConfig => browserslist_config::browserslist_config(opts),
        QueryAtom::Defaults => defaults::defaults(opts),
        QueryAtom::Dead => dead::dead(opts),
        QueryAtom::Unknown(query) => Err(Error::UnknownQuery(query.into())),
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
