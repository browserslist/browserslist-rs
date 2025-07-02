use super::{Distrib, QueryResult};
use crate::{error::Error, parser::parse_electron_version};
use browserslist_data::electron;

pub(super) fn electron_bounded_range(from: &str, to: &str) -> QueryResult {
    let from_str = from;
    let to_str = to;
    let from: f32 = parse_electron_version(from)?;
    let to: f32 = parse_electron_version(to)?;

    let versions = electron::bounded_range(from..to).map_err(|v| {
        let v = match v {
            v if v == from => from_str,
            v if v == to => to_str,
            _ => unreachable!(),
        };

        Error::UnknownElectronVersion(v.into())
    })?;

    let distribs = versions
        .iter()
        .map(|version| Distrib::new("chrome", *version))
        .collect();
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

    #[test_case("electron 0.36-1.2"; "basic")]
    #[test_case("Electron 0.37-1.0"; "case insensitive")]
    #[test_case("electron 0.37.5-1.0.3"; "with semver patch version")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test_case(
        "electron 0.1-1.2", Error::UnknownElectronVersion(String::from("0.1"));
        "unknown version 1"
    )]
    #[test_case(
        "electron 0.37-999.0", Error::UnknownElectronVersion(String::from("999.0"));
        "unknown version 2"
    )]
    #[test_case(
        "electron 1-8.a", Error::Nom(String::from("a"));
        "malformed version 1"
    )]
    #[test_case(
        "electron 1.1.1.1-2", Error::UnknownElectronVersion(String::from("1.1.1.1"));
        "malformed version 2"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::default()), error);
    }
}
