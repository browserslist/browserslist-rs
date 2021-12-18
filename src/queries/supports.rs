use super::{Distrib, QueryResult};
use crate::{data::caniuse::features::get_feature_stat, error::Error};

pub(super) fn supports(name: &str) -> QueryResult {
    if let Some(feature) = get_feature_stat(name) {
        let distribs = feature
            .iter()
            .map(|(name, version)| Distrib::new(&*name, *version))
            .collect();
        Ok(distribs)
    } else {
        Err(Error::UnknownBrowserFeature(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("supports objectrtc"; "case 1")]
    #[test_case("supports    rtcpeerconnection"; "case 2")]
    #[test_case("supports        arrow-functions"; "case 3")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("supports xxxyyyzzz", &Opts::new()),
            Error::UnknownBrowserFeature(String::from("xxxyyyzzz"))
        );
    }
}
