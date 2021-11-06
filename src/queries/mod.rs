mod browser_version_range;
mod caniuse;
mod dead;
mod defaults;
mod electron;
mod firefox_esr;
mod last_electron;
mod last_n_versions;
mod percentage;

trait Selector {
    fn select(&self, text: &str) -> Option<Vec<String>>;
}

pub fn query(query_string: &str) -> Option<Vec<String>> {
    let selectors: Vec<Box<dyn Selector>> = vec![
        Box::new(last_n_versions::LastNVersionsSelector),
        Box::new(percentage::PercentageSelector),
        Box::new(last_electron::LastElectronSelector),
        Box::new(electron::ElectronSelector),
        Box::new(browser_version_range::BrowserVersionRangeSelector),
        Box::new(firefox_esr::FirefoxESRSelector),
        Box::new(defaults::DefaultsSelector),
        Box::new(dead::DeadSelector),
    ];

    selectors
        .into_iter()
        .find_map(|selector| selector.select(query_string))
}

pub fn get_browser_alias(name: &str) -> &str {
    match name {
        "fx" | "ff" => "firefox",
        "ios" => "ios_saf",
        "explorer" => "ie",
        "blackberry" => "bb",
        "explorermobile" => "ie_mob",
        "operamini" => "op_mini",
        "operamobile" => "op_mob",
        "chromeandroid" => "and_chr",
        "firefoxandroid" => "and_ff",
        "ucandroid" => "and_uc",
        "qqandroid" => "and_qq",
        _ => name,
    }
}

#[inline]
pub fn should_filter_android(name: &str, mobile_to_desktop: bool) -> bool {
    name == "android" && !mobile_to_desktop
}

const ANDROID_EVERGREEN_FIRST: f32 = 37.0;

pub fn count_android_filter(count: usize) -> usize {
    let released = &caniuse::CANIUSE_LITE_BROWSERS
        .get("android")
        .unwrap()
        .released;
    let diff = (released.last().unwrap().parse::<f32>().unwrap()
        - ANDROID_EVERGREEN_FIRST
        - (count as f32)) as usize;
    if diff > 0 {
        1
    } else {
        1 - diff
    }
}
