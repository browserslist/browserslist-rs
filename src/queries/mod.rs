mod dead;
mod electron;
mod firefox_esr;
mod last_electron;

trait Selector {
    fn select(&self, text: &str) -> Option<Vec<String>>;
}

pub fn query(query_string: &str) -> Option<Vec<String>> {
    let queries: Vec<Box<dyn Selector>> = vec![
        Box::new(last_electron::LastElectronSelector::new()),
        Box::new(electron::ElectronSelector::new()),
        Box::new(firefox_esr::FirefoxESRSelector),
        Box::new(dead::DeadSelector),
    ];

    queries
        .into_iter()
        .find_map(|selector| selector.select(query_string))
}
