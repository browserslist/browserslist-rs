use super::{Distrib, QueryResult};
use crate::{opts::Opts, semver::Version};
use browserslist_data::{baseline, caniuse};

fn baseline_query(
    get_min_version: impl Fn(&str) -> Option<&'static str>,
    opts: &Opts,
    downstream: bool,
    kaios: bool,
) -> QueryResult {
    // Collect core browser min versions
    let mut core_mins: Vec<(&'static str, &'static str)> = Vec::new();

    let mut distribs: Vec<Distrib> = caniuse::iter_browser_stat(opts.mobile_to_desktop)
        .flat_map(|(name, version_list)| {
            let min_semver =
                get_min_version(name).map(|v| v.parse::<Version>().unwrap_or_default());
            if let Some(min_v) = get_min_version(name) {
                core_mins.push((name, min_v));
            }
            version_list
                .iter()
                .filter(move |v| {
                    let Some(ref min) = min_semver else {
                        return false;
                    };
                    v.released && v.version().parse::<Version>().unwrap_or_default() >= *min
                })
                .map(move |v| Distrib::new(name, v.version()))
        })
        .collect();

    if downstream {
        let chrome_min = core_mins.iter().find(|(b, _)| *b == "chrome").map(|(_, v)| *v);
        let firefox_min = core_mins.iter().find(|(b, _)| *b == "firefox").map(|(_, v)| *v);

        // Add Blink downstream browsers
        if let Some(chrome_min) = chrome_min {
            for ds_browser in baseline::blink_downstream_browsers() {
                if ds_browser == "kaios" {
                    continue;
                }
                if let Some(min_v) = baseline::get_downstream_blink_min_version(chrome_min, ds_browser) {
                    let min_semver = min_v.parse::<Version>().unwrap_or_default();
                    if let Some((_, version_list)) =
                        caniuse::get_browser_stat(ds_browser, opts.mobile_to_desktop)
                    {
                        for v in version_list.iter().filter(|v| {
                            v.released
                                && v.version().parse::<Version>().unwrap_or_default() >= min_semver
                        }) {
                            distribs.push(Distrib::new(ds_browser, v.version()));
                        }
                    }
                }
            }
        }

        // Add Gecko downstream browsers (KaiOS requires explicit opt-in)
        if let Some(firefox_min) = firefox_min {
            for ds_browser in baseline::gecko_downstream_browsers() {
                if ds_browser == "kaios" && !kaios {
                    continue;
                }
                if let Some(min_v) =
                    baseline::get_downstream_gecko_min_version(firefox_min, ds_browser)
                {
                    let min_semver = min_v.parse::<Version>().unwrap_or_default();
                    if let Some((_, version_list)) =
                        caniuse::get_browser_stat(ds_browser, opts.mobile_to_desktop)
                    {
                        for v in version_list.iter().filter(|v| {
                            v.released
                                && v.version().parse::<Version>().unwrap_or_default() >= min_semver
                        }) {
                            distribs.push(Distrib::new(ds_browser, v.version()));
                        }
                    }
                }
            }
        }
    }

    Ok(distribs)
}

pub(super) fn baseline_widely(opts: &Opts, downstream: bool, kaios: bool) -> QueryResult {
    baseline_query(baseline::get_baseline_widely_min_version, opts, downstream, kaios)
}

pub(super) fn baseline_newly(opts: &Opts, downstream: bool, kaios: bool) -> QueryResult {
    baseline_query(baseline::get_baseline_newly_min_version, opts, downstream, kaios)
}

pub(super) fn baseline_year(year: u16, opts: &Opts, downstream: bool, kaios: bool) -> QueryResult {
    baseline_query(
        |browser| baseline::get_baseline_year_min_version(year, browser),
        opts,
        downstream,
        kaios,
    )
}

/// Compute cutoff = widelyAvailableOnDate - 30 months, then query by that cutoff date.
pub(super) fn baseline_widely_on_date(
    date: &str,
    opts: &Opts,
    downstream: bool,
    kaios: bool,
) -> QueryResult {
    let cutoff = subtract_30_months(date)?;
    baseline_query(
        |browser| baseline::get_baseline_cutoff_date_min_version(&cutoff, browser),
        opts,
        downstream,
        kaios,
    )
}

/// Subtract 30 months from a YYYY-MM-DD date string.
fn subtract_30_months(date: &str) -> Result<String, crate::error::Error> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(crate::error::Error::UnknownQuery(
            format!("invalid date: {date}").into(),
        ));
    }
    let year: i32 = parts[0].parse().unwrap_or(0);
    let month: i32 = parts[1].parse().unwrap_or(1);
    let day = parts[2];

    // Subtract 30 months
    let total_months = year * 12 + (month - 1) - 30;
    let new_year = total_months / 12;
    let new_month = total_months % 12 + 1;

    Ok(format!("{:04}-{:02}-{}", new_year, new_month, day))
}

#[cfg(test)]
mod tests {
    use super::*;
    use browserslist_data::baseline;
    use std::collections::HashMap;
    use std::process::Command;
    use test_case::test_case;

    /// Run a Node.js snippet and return stdout lines.
    fn node_output(script: &str) -> Vec<String> {
        let out = Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .expect("node must be available");
        assert!(out.status.success(), "node failed: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Call getCompatibleVersions() and return a map of caniuse_browser -> min_version.
    fn js_min_versions(js_opts: &str) -> HashMap<String, String> {
        let opts_str = if js_opts.is_empty() {
            "suppressWarnings: true".to_owned()
        } else {
            format!("{js_opts}, suppressWarnings: true")
        };
        let script = format!(
            r#"
const {{ getCompatibleVersions }} = require('./node_modules/baseline-browser-mapping');
const map = {{chrome:'chrome',chrome_android:'and_chr',edge:'edge',firefox:'firefox',firefox_android:'and_ff',safari:'safari',safari_ios:'ios_saf'}};
const r = getCompatibleVersions({{ {opts_str} }});
r.filter(e=>map[e.browser]).forEach(e => console.log(map[e.browser] + ' ' + e.version));
"#
        );
        node_output(&script)
            .into_iter()
            .map(|line| {
                let (b, v) = line.split_once(' ').unwrap();
                (b.to_owned(), v.to_owned())
            })
            .collect()
    }

    /// Verify that the minimum version our data returns for each browser matches
    /// what getCompatibleVersions() from the JS library reports.
    fn assert_min_versions_match(
        label: &str,
        expected: &HashMap<String, String>,
        get_min: impl Fn(&str) -> Option<&'static str>,
    ) {
        for (browser, js_version) in expected {
            let rust_version = get_min(browser).unwrap_or_else(|| {
                panic!("[{label}] {browser}: Rust returned None, JS expected {js_version}")
            });
            let js_v: Version = js_version.parse().unwrap_or_default();
            let rust_v: Version = rust_version.parse().unwrap_or_default();
            assert_eq!(
                js_v, rust_v,
                "[{label}] {browser}: Rust min={rust_version}, JS min={js_version}"
            );
        }
        assert_eq!(expected.len(), 7, "[{label}] expected 7 core browsers, got {}", expected.len());
    }

    #[test]
    fn widely_min_versions_match_js() {
        let expected = js_min_versions("");
        assert_min_versions_match("widely", &expected, |b| {
            baseline::get_baseline_widely_min_version(b)
        });
    }

    #[test_case(2015; "year_2015")]
    #[test_case(2016; "year_2016")]
    #[test_case(2017; "year_2017")]
    #[test_case(2018; "year_2018")]
    #[test_case(2019; "year_2019")]
    #[test_case(2020; "year_2020")]
    #[test_case(2021; "year_2021")]
    #[test_case(2022; "year_2022")]
    #[test_case(2023; "year_2023")]
    #[test_case(2024; "year_2024")]
    fn year_min_versions_match_js(year: u16) {
        let expected = js_min_versions(&format!("targetYear: {year}"));
        assert_min_versions_match(&format!("year {year}"), &expected, |b| {
            baseline::get_baseline_year_min_version(year, b)
        });
    }

    #[test_case("2023-04-05"; "date_2023_04_05")]
    #[test_case("2021-01-01"; "date_2021_01_01")]
    #[test_case("2024-06-15"; "date_2024_06_15")]
    fn widely_on_date_min_versions_match_js(date: &str) {
        let expected = js_min_versions(&format!("widelyAvailableOnDate: '{date}'"));
        let cutoff = subtract_30_months(date).unwrap();
        assert_min_versions_match(&format!("date {date}"), &expected, |b| {
            baseline::get_baseline_cutoff_date_min_version(&cutoff, b)
        });
    }

    #[test_case("baseline widely available"; "widely")]
    #[test_case("baseline 2020"; "year 2020")]
    #[test_case("baseline 2022"; "year 2022")]
    #[test_case("baseline 2015"; "year 2015")]
    #[test_case("BASELINE WIDELY AVAILABLE"; "case insensitive widely")]
    #[test_case("baseline widely available on 2023-04-05"; "widely on date")]
    fn valid_queries_return_results(query: &str) {
        let results = crate::resolve([query], &Opts::default()).unwrap();
        assert!(!results.is_empty(), "expected non-empty results for {query:?}");
    }

    #[test_case("baseline newly available"; "newly")]
    #[test_case("Baseline Newly Available"; "case insensitive newly")]
    fn valid_queries_no_error(query: &str) {
        // "newly" min versions may be ahead of the caniuse data, so results can be empty
        crate::resolve([query], &Opts::default()).expect("query should not error");
    }

    #[test_case("baseline 2022 with downstream"; "year_downstream")]
    #[test_case("baseline widely available with downstream"; "widely_downstream")]
    fn valid_queries_with_downstream(query: &str) {
        let results = crate::resolve([query], &Opts::default()).unwrap();
        assert!(!results.is_empty(), "expected non-empty results for {query:?}");
        // Should have more results than without downstream
        let without = crate::resolve(
            [query.replace(" with downstream", "").as_str()],
            &Opts::default(),
        )
        .unwrap();
        assert!(
            results.len() > without.len(),
            "downstream should add browsers: got {}, without got {}",
            results.len(),
            without.len()
        );
    }

    #[test_case("baseline 2020 with downstream including kaios"; "year_downstream_kaios")]
    fn valid_queries_with_kaios(query: &str) {
        let results = crate::resolve([query], &Opts::default()).unwrap();
        assert!(!results.is_empty(), "expected non-empty results for {query:?}");
        // Should include kaios
        assert!(
            results.iter().any(|d| d.name() == "kaios"),
            "expected kaios in results"
        );
    }
}
