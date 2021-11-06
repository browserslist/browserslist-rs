use crate::{data::caniuse, opts::Opts};

mod browser_version_range;
mod dead;
mod defaults;
mod electron;
mod firefox_esr;
mod last_electron;
mod last_n_versions;
mod percentage;
mod phantom;

trait Selector {
    fn select(&self, text: &str, opts: &Opts) -> Option<Vec<String>>;
}

pub fn query(query_string: &str, opts: &Opts) -> Option<Vec<String>> {
    let selectors: Vec<Box<dyn Selector>> = vec![
        Box::new(last_n_versions::LastNVersionsSelector),
        Box::new(percentage::PercentageSelector),
        Box::new(last_electron::LastElectronSelector),
        Box::new(electron::ElectronSelector),
        Box::new(browser_version_range::BrowserVersionRangeSelector),
        Box::new(firefox_esr::FirefoxESRSelector),
        Box::new(phantom::PhantomSelector),
        Box::new(defaults::DefaultsSelector),
        Box::new(dead::DeadSelector),
    ];

    selectors
        .into_iter()
        .find_map(|selector| selector.select(query_string, opts))
}

#[inline]
pub fn should_filter_android(name: &str, mobile_to_desktop: bool) -> bool {
    name == "android" && !mobile_to_desktop
}

pub fn count_android_filter(count: usize, mobile_to_desktop: bool) -> usize {
    let released = &caniuse::get_browser_stat("android", mobile_to_desktop)
        .unwrap()
        .released;
    let diff = (released.last().unwrap().parse::<f32>().unwrap()
        - caniuse::ANDROID_EVERGREEN_FIRST
        - (count as f32)) as usize;
    if diff > 0 {
        1
    } else {
        1 - diff
    }
}
