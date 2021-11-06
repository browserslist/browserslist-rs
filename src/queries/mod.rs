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
