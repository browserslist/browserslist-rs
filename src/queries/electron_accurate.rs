use super::{Distrib, QueryResult};
use crate::{
    data::electron::{self, parse_version},
    error::Error,
};

pub(super) fn electron_accurate(version: &str) -> QueryResult {
    let version_str = version;
    let version: f32 = parse_version(version)?;

    let distribs = electron::get(version)
        .map(|chromium_version| vec![Distrib::new("chrome", chromium_version)])
        .ok_or_else(|| Error::UnknownElectronVersion(version_str.to_string()))?;
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("electron 1.1"; "basic")]
    #[test_case("electron 4.0.4"; "with semver patch version")]
    #[test_case("Electron 1.1"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test_case(
        "electron 0.19", Error::UnknownElectronVersion(String::from("0.19"));
        "unknown version"
    )]
    #[test_case(
        "electron 8.a", Error::Nom(String::from("a"));
        "malformed version 1"
    )]
    #[test_case(
        "electron 1.1.1.1", Error::UnknownElectronVersion(String::from("1.1.1.1"));
        "malformed version 2"
    )]
    #[test_case(
        "electron 7.01", Error::UnknownElectronVersion(String::from("7.01"));
        "malformed version 3"
    )]
    #[test_case(
        "electron 999.0", Error::UnknownElectronVersion(String::from("999.0"));
        "malformed version 4"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::default()), error);
    }
}
