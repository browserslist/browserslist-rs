use super::{Distrib, QueryResult};
use crate::parser::Comparator;
use browserslist_data::caniuse;

pub(super) fn percentage(comparator: Comparator, popularity: f32) -> QueryResult {
    let distribs = caniuse::iter_browser_stat(false)
        .flat_map(|(name, list)| {
            list.iter()
                .filter(|version| {
                    let usage = version.global_usage;
                    match comparator {
                        Comparator::Greater => usage > popularity,
                        Comparator::GreaterOrEqual => usage >= popularity,
                        Comparator::Less => usage < popularity,
                        Comparator::LessOrEqual => usage <= popularity,
                    }
                })
                .map(move |version| Distrib::new(name, version.version()))
        })
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use browserslist_data::caniuse;
    use std::{collections::HashMap, process::Command};
    use test_case::test_case;

    #[test_case("> 10%"; "greater")]
    #[test_case(">= 5%"; "greater or equal")]
    #[test_case("< 5%"; "less")]
    #[test_case("<= 5%"; "less or equal")]
    #[test_case(">10%"; "no space")]
    #[test_case("> 10.2%"; "with float")]
    #[test_case("> .2%"; "with float that has a leading dot")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test]
    fn global_usage_matches_caniuse() {
        let output = Command::new("node")
            .args([
                "-p",
                "JSON.stringify(require('browserslist').usage.global)",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let expected: HashMap<String, f64> = serde_json::from_slice(&output.stdout).unwrap();
        let actual = caniuse::iter_global_usage()
            .map(|(name, version, usage)| (format!("{name} {version}"), usage))
            .collect::<HashMap<_, _>>();

        for (key, expected) in expected {
            let actual = actual[&key];
            let tolerance = f32::EPSILON * 2.0 * (expected as f32).max(1.0);
            assert!(
                (actual - expected as f32).abs() <= tolerance,
                "{key}: expected {expected}, got {actual}"
            );
        }
    }
}
