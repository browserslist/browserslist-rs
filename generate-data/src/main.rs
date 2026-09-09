use anyhow::Result;
use chrono::Datelike;
use indexmap::IndexMap;
use miniz_oxide::deflate::compress_to_vec;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::Write,
};

const OUT_DIR: &str = "data/src/generated";
const GLOBAL_USAGE_SCALE: u32 = 1_000;
const REGION_USAGE_SCALE: u32 = 100_000;

fn encode_browser_name(name: &str) -> u8 {
    match name {
        "ie" => 1,
        "edge" => 2,
        "firefox" => 3,
        "chrome" => 4,
        "safari" => 5,
        "opera" => 6,
        "ios_saf" => 7,
        "op_mini" => 8,
        "android" => 9,
        "bb" => 10,
        "op_mob" => 11,
        "and_chr" => 12,
        "and_ff" => 13,
        "ie_mob" => 14,
        "and_uc" => 15,
        "samsung" => 16,
        "and_qq" => 17,
        "baidu" => 18,
        "kaios" => 19,
        _ => unreachable!("unknown browser name"),
    }
}

#[derive(Deserialize)]
struct Caniuse {
    agents: BTreeMap<String, Agent>,
    data: BTreeMap<String, Feature>,
}

#[derive(Deserialize)]
struct Agent {
    version_list: Vec<VersionDetail>,
}

#[derive(Clone, Deserialize, Serialize)]
struct VersionDetail {
    version: String,
    global_usage: f32,
    release_date: Option<i64>,
}

#[derive(Deserialize)]
struct Feature {
    stats: BTreeMap<String, IndexMap<String, String>>,
}

fn main() -> Result<()> {
    build_info()?;
    build_electron_to_chromium()?;
    build_node_versions()?;
    build_node_release_schedule()?;

    let mut strpool = StrPool::default();
    build_caniuse(&mut strpool)?;
    build_baseline(&mut strpool)?;
    fs::write(format!("{OUT_DIR}/strpool.bin"), strpool.pool.as_bytes())?;

    Ok(())
}

fn build_info() -> Result<()> {
    use std::process::{Command, Stdio};

    let mut infofile = fs::File::create(format!("{OUT_DIR}/info.txt"))?;

    let output = Command::new("git")
        .arg("submodule")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .output()?;

    if output.status.success() {
        infofile.write_all(&output.stdout)?;
        Ok(())
    } else {
        anyhow::bail!("git submodule failed: {:?}", output.status.code())
    }
}

fn build_electron_to_chromium() -> Result<()> {
    let path = format!("{OUT_DIR}/electron-to-chromium.rs");

    let mut data = serde_json::from_slice::<BTreeMap<String, String>>(&fs::read(
        "node_modules/electron-to-chromium/versions.json",
    )?)?
    .into_iter()
    .map(|(electron_version, chromium_version)| {
        (electron_version.parse::<f32>().unwrap(), chromium_version)
    })
    .collect::<Vec<_>>();
    data.sort_by(|(a, _), (b, _)| a.total_cmp(b));
    let (electron_versions, chromium_versions): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    let code = quote! {
        static ELECTRON_VERSIONS: &[f32] = &[ #(#electron_versions),* ];
        static CHROMIUM_VERSIONS: &[&str] = &[ #(#chromium_versions),* ];
    };

    fs::write(path, code.to_string())?;

    Ok(())
}

fn build_node_versions() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        version: String,
    }

    let path = format!("{OUT_DIR}/node-versions.rs");

    let releases: Vec<NodeRelease> = serde_json::from_slice(&fs::read(
        "node_modules/node-releases/data/processed/envs.json",
    )?)?;

    let versions = releases.into_iter().map(|release| release.version);
    fs::write(
        path,
        quote! {
            static NODE_VERSIONS: &[&str] = &[#(#versions),*];
        }
        .to_string(),
    )?;

    Ok(())
}

fn build_node_release_schedule() -> Result<()> {
    use chrono::{Datelike, NaiveDate};

    #[derive(Deserialize)]
    struct NodeRelease {
        start: String,
        end: String,
    }

    let path = format!("{OUT_DIR}/node-release-schedule.rs");

    let schedule: BTreeMap<String, NodeRelease> = serde_json::from_slice(&fs::read(
        "node_modules/node-releases/data/release-schedule/release-schedule.json",
    )?)?;
    let mut versions = schedule
        .into_iter()
        .map(|(version, NodeRelease { start, end })| {
            let date_format = "%Y-%m-%d";
            let start = NaiveDate::parse_from_str(&start, date_format)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let end = NaiveDate::parse_from_str(&end, date_format)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();

            (version, (start, end))
        })
        .collect::<Vec<_>>();
    // filter by end date to quickly reduce scope
    versions.sort_by_key(|(_, (_, end))| *end);

    let (versions, dates): (Vec<_>, Vec<_>) = versions
        .into_iter()
        .map(|(version, (start, end))| {
            let version = version.trim_start_matches('v');

            let start_year = start.year();
            let start_month = start.month();
            let start_day = start.day();
            let end_year = end.year();
            let end_month = end.month();
            let end_day = end.day();

            let start = quote! { chrono::NaiveDate::from_ymd_opt(#start_year, #start_month, #start_day).unwrap() };
            let end = quote! { chrono::NaiveDate::from_ymd_opt(#end_year, #end_month, #end_day).unwrap() };

            (version.to_owned(), (start, end))
        })
        .unzip();
    let starts = dates.iter().map(|(start, _)| start);
    let ends = dates.iter().map(|(_, end)| end);

    fs::write(
        path,
        quote! {
            static NODE_RELEASE_VERSIONS: &[&str] = &[#(#versions),*];
            static NODE_RELEASE_START: &[chrono::NaiveDate] = &[#(#starts),*];
            static NODE_RELEASE_END: &[chrono::NaiveDate] = &[#(#ends),*];
        }
        .to_string(),
    )?;

    Ok(())
}

fn build_caniuse(strpool: &mut StrPool) -> Result<()> {
    let data = parse_caniuse_global()?;
    let region_data = parse_caniuse_regions()?;

    // caniuse browsers
    {
        let mut version_ids = Vec::new();
        let mut release_years = Vec::new();
        let mut release_months = Vec::new();
        let mut release_days = Vec::new();
        let mut released = Vec::new();
        let mut global_usage = Vec::new();
        let mut stats = Vec::new();

        for (name, agent) in &data.agents {
            let name_str_id = strpool.insert(name);
            let start: u32 = version_ids.len().try_into().unwrap();

            for version in &agent.version_list {
                let version_str_id = strpool.insert(&version.version);
                let usage = u16::try_from(quantize_usage(
                    version.global_usage,
                    GLOBAL_USAGE_SCALE,
                    true,
                )?)?;
                let (year, month, day) = match version.release_date {
                    Some(timestamp) => {
                        let date =
                            chrono::DateTime::from_timestamp(timestamp, 0).ok_or_else(|| {
                                anyhow::anyhow!("invalid release timestamp: {timestamp}")
                            })?;
                        (
                            u8::try_from(date.year() - 1970)?,
                            u8::try_from(date.month())?,
                            u8::try_from(date.day())?,
                        )
                    }
                    None => (0, 0, 0),
                };
                let is_released = version.release_date.is_some();

                version_ids.push(version_str_id);
                release_years.push(year);
                release_months.push(month);
                release_days.push(day);
                released.push(u8::from(is_released));
                global_usage.push(usage);
            }

            let end: u32 = version_ids.len().try_into().unwrap();
            stats.push((name_str_id, start, end));
        }

        stats.sort_by_key(|(name_str_id, ..)| strpool.get(*name_str_id));
        let stat_keys = stats.iter().map(|(key, ..)| *key).collect::<Vec<_>>();
        let stat_starts = stats
            .iter()
            .map(|(.., start, _)| *start)
            .collect::<Vec<_>>();
        let stat_ends = stats.iter().map(|(.., _, end)| *end).collect::<Vec<_>>();

        let version_ids =
            write_u32_array("caniuse-version-ids", "VERSION_LIST_VERSION", &version_ids)?;
        let release_years = write_blob("VERSION_LIST_RELEASE_YEAR", &release_years)?;
        let release_months = write_blob("VERSION_LIST_RELEASE_MONTH", &release_months)?;
        let release_days = write_blob("VERSION_LIST_RELEASE_DAY", &release_days)?;
        let release_dates = quote! {
            #release_years #release_months #release_days
            static VERSION_LIST_RELEASE_DATE: LazyLock<Vec<i64>> = LazyLock::new(||
                crate::decode_release_dates(
                    &*VERSION_LIST_RELEASE_YEAR,
                    &*VERSION_LIST_RELEASE_MONTH,
                    &*VERSION_LIST_RELEASE_DAY,
                ));
        };
        let global_usage = write_u16_array(
            "caniuse-global-usage",
            "VERSION_LIST_GLOBAL_USAGE",
            &global_usage,
        )?;
        let stat_keys = write_u32_array(
            "caniuse-browser-stat-keys",
            "BROWSERS_STATS_KEY",
            &stat_keys,
        )?;
        let stat_starts = write_u32_array(
            "caniuse-browser-stat-starts",
            "BROWSERS_STATS_START",
            &stat_starts,
        )?;
        let stat_ends = write_u32_array(
            "caniuse-browser-stat-ends",
            "BROWSERS_STATS_END",
            &stat_ends,
        )?;
        let released = write_blob("caniuse-released.bin", "VERSION_LIST_RELEASED", &released)?;

        fs::write(
            format!("{OUT_DIR}/caniuse-browsers.rs"),
            quote! {
                #version_ids
                #release_dates
                #released
                #global_usage
                #stat_keys
                #stat_starts
                #stat_ends
            }
            .to_string(),
        )?;
    }

    // caniuse features
    {
        let mut features = Vec::new();
        let mut stats = Vec::new();
        let mut flags: Vec<u8> = Vec::new();

        for (name, feature) in &data.data {
            let start = stats.len();
            // `feature.stats` is keyed by browser name, so the browsers of one feature
            // already come out in the order `Feature::get` binary searches for.
            for (browser, ver) in &feature.stats {
                let agent = data.agents.get(browser).ok_or_else(|| {
                    anyhow::anyhow!("feature `{name}`: unknown browser `{browser}`")
                })?;
                // caniuse states every feature for every version of a browser, so no
                // versions are stored here at all: the flags line up with the browser's
                // own version list, position by position.
                anyhow::ensure!(
                    agent.version_list.len() == ver.len()
                        && agent
                            .version_list
                            .iter()
                            .all(|version| ver.contains_key(&version.version)),
                    "feature `{name}` does not cover every version of `{browser}`"
                );

                flags.extend(agent.version_list.iter().map(|version| {
                    let support = &ver[&version.version];
                    let mut bit: u8 = 0;
                    if support.contains('y') {
                        bit |= 1;
                    }
                    if support.contains('a') {
                        bit |= 2;
                    }
                    bit
                }));
                stats.push(browser.as_str());
            }
            let end = stats.len();
            anyhow::ensure!(
                stats[start..end].is_sorted(),
                "feature `{name}`: browsers must be in name order"
            );

            let name_str_id = strpool.insert(name);
            features.push((name_str_id, start, end));
        }

        features.sort_by_key(|(name, ..)| strpool.get(*name));

        let stats_name = stats
            .iter()
            .map(|browser| encode_browser_name(browser))
            .collect::<Vec<_>>();
        let feature_keys = features.iter().map(|(key, ..)| *key).collect::<Vec<_>>();
        let feature_starts = features
            .iter()
            .map(|(.., start, _)| u32::try_from(*start))
            .collect::<Result<Vec<_>, _>>()?;
        let feature_ends = features
            .iter()
            .map(|(.., _, end)| u32::try_from(*end))
            .collect::<Result<Vec<_>, _>>()?;

        let flags = write_blob("caniuse-feature-flags.bin", "FEATURES_STAT_FLAGS", &flags)?;
        let stats_name = write_blob(
            "caniuse-feature-browsers.bin",
            "FEATURES_STAT_BROWSERS",
            &stats_name,
        )?;
        let feature_keys = write_u32_array("caniuse-feature-keys", "FEATURES_KEY", &feature_keys)?;
        let feature_starts =
            write_u32_array("caniuse-feature-starts", "FEATURES_START", &feature_starts)?;
        let feature_ends = write_u32_array("caniuse-feature-ends", "FEATURES_END", &feature_ends)?;

        fs::write(
            format!("{OUT_DIR}/caniuse-feature-matching.rs"),
            quote! {
                #feature_keys
                #feature_starts
                #feature_ends
                #flags
                #stats_name
            }
            .to_string(),
        )?;
    }

    // caniuse region
    {
        let mut usages = Vec::new();
        let mut region_usages = Vec::new();

        for (region_name, browsers) in &region_data {
            let start = usages.len();
            for (name, stat) in browsers {
                let agent = data.agents.get(name).unwrap();
                for (version, &usage) in stat {
                    let version = if version.as_str() == "0" {
                        &agent.version_list.last().unwrap().version
                    } else {
                        version
                    };

                    let version_str_id = strpool.insert(version);
                    usages.push((encode_browser_name(name), version_str_id, usage));
                }
            }
            let end = usages.len();
            // Keep every region in the same browser/version order.  Besides making
            // the browser and version columns more compressible, this is independent
            // of the region's usage values.  `RegionData::iter` restores the public
            // usage-descending order for callers.
            usages[start..end].sort_by(
                |(left_browser, left_version, _), (right_browser, right_version, _)| {
                    left_browser
                        .cmp(right_browser)
                        .then_with(|| strpool.get(*left_version).cmp(strpool.get(*right_version)))
                },
            );

            let region_str_id = strpool.insert(region_name);
            region_usages.push((region_str_id, start, end));
        }

        region_usages.sort_by_key(|(region, ..)| strpool.get(*region));

        let browsers = write_blob(
            "caniuse-region-browsers.bin",
            "REGIONS_BROWSERS",
            &usages.iter().map(|(b, ..)| *b).collect::<Vec<_>>(),
        )?;

        let versions = usages.iter().map(|(_, v, _)| *v).collect::<Vec<_>>();
        let region_usage_values = usages
            .iter()
            .map(|(_, _, usage)| quantize_usage(*usage, REGION_USAGE_SCALE, false))
            .collect::<Result<Vec<_>>>()?;
        let region_keys = region_usages
            .iter()
            .map(|(key, ..)| *key)
            .collect::<Vec<_>>();
        let region_starts = region_usages
            .iter()
            .map(|(.., start, _)| u32::try_from(*start))
            .collect::<Result<Vec<_>, _>>()?;
        let region_ends = region_usages
            .iter()
            .map(|(.., _, end)| u32::try_from(*end))
            .collect::<Result<Vec<_>, _>>()?;
        let versions = write_u32_array("caniuse-region-versions", "REGIONS_VERSIONS", &versions)?;
        let region_usages = write_u32_array(
            "caniuse-region-usages",
            "REGIONS_USAGES",
            &region_usage_values,
        )?;
        let region_keys = write_u32_array("caniuse-region-keys", "REGIONS_KEY", &region_keys)?;
        let region_starts =
            write_u32_array("caniuse-region-starts", "REGIONS_START", &region_starts)?;
        let region_ends = write_u32_array("caniuse-region-ends", "REGIONS_END", &region_ends)?;

        fs::write(
            format!("{OUT_DIR}/caniuse-region-matching.rs"),
            quote! {
                #region_keys
                #region_starts
                #region_ends
                #browsers
                #versions
                #region_usages
            }
            .to_string(),
        )?;
    }

    Ok(())
}

fn parse_caniuse_global() -> Result<Caniuse> {
    let json = run_node(
        r#"
const { agents } = require('./node_modules/caniuse-lite/dist/unpacker/agents');
const featuresMap = require('./node_modules/caniuse-lite/data/features');
const unpackFeature = require('./node_modules/caniuse-lite/dist/unpacker/feature');
const result = { agents: {}, data: {} };
for (const [name, agent] of Object.entries(agents)) {
    const releaseDate = agent.release_date || {};
    result.agents[name] = {
        version_list: (agent.versions || []).filter(v => v != null).map(v => ({
            version: v,
            global_usage: agent.usage_global[v] || 0,
            release_date: releaseDate[v] ?? null
        }))
    };
}
for (const [name, featureModule] of Object.entries(featuresMap)) {
    const unpacked = unpackFeature(featureModule);
    result.data[name] = { stats: unpacked.stats };
}
process.stdout.write(JSON.stringify(result));
"#,
    )?;
    Ok(serde_json::from_str(&json)?)
}

fn parse_caniuse_regions() -> Result<BTreeMap<String, BTreeMap<String, BTreeMap<String, f32>>>> {
    let json = run_node(
        r#"
const fs = require('fs');
const path = require('path');
const browsersMap = require('./node_modules/caniuse-lite/data/browsers');
const regionDir = './node_modules/caniuse-lite/data/regions';
const files = fs.readdirSync(regionDir).filter(f => f.endsWith('.js')).sort();
const result = {};
for (const file of files) {
    const name = path.basename(file, '.js');
    const packed = require(path.resolve(regionDir, file));
    const regionData = {};
    for (const [browserKey, data] of Object.entries(packed)) {
        const browserName = browsersMap[browserKey];
        if (!browserName) continue;
        const entries = [...Object.entries(data)].filter(([version]) => version !== '_');
        if (entries.length > 0) { regionData[browserName] = Object.fromEntries(entries); }
    }
    result[name] = regionData;
}
process.stdout.write(JSON.stringify(result));
"#,
    )?;
    Ok(serde_json::from_str(&json)?)
}

fn build_baseline(strpool: &mut StrPool) -> Result<()> {
    #[derive(Deserialize)]
    struct TimelineEvent {
        date: String,
        browsers: Vec<TimelineBrowserVersion>,
    }

    #[derive(Deserialize)]
    struct TimelineBrowserVersion {
        browser: String,
        version: String,
    }

    let json = run_node(
        r#"
const bbm = require('baseline-browser-mapping');
const timeline = bbm.getTimeline({
    listAllBrowsers: true,
    includeDownstreamBrowsers: true,
    includeKaiOS: true,
});
process.stdout.write(JSON.stringify(timeline));
"#,
    )?;
    let timeline: Vec<TimelineEvent> = serde_json::from_str(&json)?;

    // Map baseline-browser-mapping browser names to caniuse browser names,
    // dropping browsers that caniuse doesn't track (as browserslist does).
    let browser_map: HashMap<&str, &str> = [
        ("chrome", "chrome"),
        ("chrome_android", "and_chr"),
        ("edge", "edge"),
        ("firefox", "firefox"),
        ("firefox_android", "and_ff"),
        ("safari", "safari"),
        ("safari_ios", "ios_saf"),
        ("webview_android", "android"),
        ("samsunginternet_android", "samsung"),
        ("opera_android", "op_mob"),
        ("opera", "opera"),
        ("qq_android", "and_qq"),
        ("uc_android", "and_uc"),
        ("kai_os", "kaios"),
    ]
    .into_iter()
    .collect();

    let mut browsers = BTreeSet::new();
    let mut events: Vec<(&str, BTreeMap<&str, &str>)> = Vec::new();
    for event in &timeline {
        if let Some((last_date, _)) = events.last() {
            anyhow::ensure!(*last_date < event.date.as_str(), "timeline must be sorted");
        }
        let snapshot: BTreeMap<&str, &str> = event
            .browsers
            .iter()
            .filter_map(|entry| {
                browser_map
                    .get(entry.browser.as_str())
                    .map(|name| (*name, entry.version.as_str()))
            })
            .collect();
        browsers.extend(snapshot.keys().copied());
        // Events can become identical after dropping unmapped browsers.
        match events.last() {
            Some((_, last)) if *last == snapshot => {}
            _ => events.push((event.date.as_str(), snapshot)),
        }
    }

    // Flatten the snapshots into one array of (browser id, pooled version)
    // and index into it per event, keyed by the date as a decimal `yyyymmdd`.
    // This keeps the tables free of pointers (and their relocations).
    let mut version_entries: Vec<(u8, u32)> = Vec::new();
    let mut timeline_entries: Vec<(u32, u16, u16)> = Vec::new();
    for (date, snapshot) in &events {
        let date: u32 = date.replace('-', "").parse()?;
        let start: u16 = version_entries.len().try_into()?;
        version_entries.extend(
            snapshot
                .iter()
                .map(|(browser, version)| (encode_browser_name(browser), strpool.insert(version))),
        );
        let end: u16 = version_entries.len().try_into()?;
        timeline_entries.push((date, start, end));
    }

    let browser_tokens = browsers.iter().map(|name| {
        let id = encode_browser_name(name);
        quote! { #id }
    });
    let version_browsers = version_entries
        .iter()
        .map(|(browser, _)| *browser)
        .collect::<Vec<_>>();
    let version_versions = version_entries
        .iter()
        .map(|(_, version)| *version)
        .collect::<Vec<_>>();
    let timeline_dates = timeline_entries
        .iter()
        .map(|(date, ..)| *date)
        .collect::<Vec<_>>();
    let timeline_starts = timeline_entries
        .iter()
        .map(|(.., start, _)| *start)
        .collect::<Vec<_>>();
    let timeline_ends = timeline_entries
        .iter()
        .map(|(.., _, end)| *end)
        .collect::<Vec<_>>();
    let version_browsers = write_blob(
        "baseline-version-browsers.bin",
        "BASELINE_VERSION_BROWSER",
        &version_browsers,
    )?;
    let version_versions = write_u32_array(
        "baseline-version-versions",
        "BASELINE_VERSION_VERSION",
        &version_versions,
    )?;
    let timeline_dates = write_u32_array(
        "baseline-timeline-dates",
        "BASELINE_TIMELINE_DATE",
        &timeline_dates,
    )?;
    let timeline_starts = write_u16_array(
        "baseline-timeline-starts",
        "BASELINE_TIMELINE_START",
        &timeline_starts,
    )?;
    let timeline_ends = write_u16_array(
        "baseline-timeline-ends",
        "BASELINE_TIMELINE_END",
        &timeline_ends,
    )?;
    fs::write(
        format!("{OUT_DIR}/baseline.rs"),
        quote! {
            const BASELINE_BROWSERS: &[u8] = &[#(#browser_tokens),*];
            #version_browsers
            #version_versions
            #timeline_dates
            #timeline_starts
            #timeline_ends
        }
        .to_string(),
    )?;

    Ok(())
}

/// Writes a byte array twice: verbatim as `<name>`, and as a raw deflate stream in
/// `<name>.deflate`.
///
/// The returned expression selects one input at compile time; both representations
/// are deliberately published so consumers can choose with `deflate`.
fn write_blob(name: &str, static_name: &str, bytes: &[u8]) -> Result<TokenStream> {
    fs::write(format!("{OUT_DIR}/{name}"), bytes)?;

    let deflated = compress_to_vec(bytes, 10);
    fs::write(format!("{OUT_DIR}/{name}.deflate"), deflated)?;

    let deflate_name = format!("{name}.deflate");
    let static_name = format_ident!("{static_name}");
    Ok(quote! {
        #[cfg(feature = "deflate")]
        static #static_name: LazyLock<Vec<u8>> =
            LazyLock::new(|| crate::inflate(include_bytes!(#deflate_name)));
        #[cfg(not(feature = "deflate"))]
        const #static_name: &[u8] = include_bytes!(#name);
    })
}

/// Writes a numeric column as a `LazyLock<Vec<_>>`. The closure either copies the
/// literal values or inflates and decodes their little-endian byte representation.
fn write_array(
    name: &str,
    static_name: &str,
    bytes: Vec<u8>,
    values: Vec<TokenStream>,
    value_type: TokenStream,
) -> Result<TokenStream> {
    let deflated = compress_to_vec(&bytes, 10);
    let deflate_name = format!("{name}.deflate");
    let static_name = format_ident!("{static_name}");
    fs::write(format!("{OUT_DIR}/{deflate_name}"), deflated)?;
    Ok(quote! {
        #[cfg(feature = "deflate")]
        static #static_name: LazyLock<Vec<#value_type>> = LazyLock::new(|| {
            crate::inflate(include_bytes!(#deflate_name))
                .as_chunks::<{ std::mem::size_of::<#value_type>() }>()
                .0
                .iter()
                .map(|bytes| #value_type::from_le_bytes(*bytes))
                .collect()
        });
        #[cfg(not(feature = "deflate"))]
        static #static_name: &[#value_type] = &[#(#values),*];
    })
}

fn write_u16_array(name: &str, static_name: &str, values: &[u16]) -> Result<TokenStream> {
    write_array(
        name,
        static_name,
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
        values.iter().map(|value| quote! { #value }).collect(),
        quote! { u16 },
    )
}

fn write_u32_array(name: &str, static_name: &str, values: &[u32]) -> Result<TokenStream> {
    write_array(
        name,
        static_name,
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
        values.iter().map(|value| quote! { #value }).collect(),
        quote! { u32 },
    )
}

fn write_i64_array(name: &str, static_name: &str, values: &[i64]) -> Result<TokenStream> {
    write_array(
        name,
        static_name,
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
        values.iter().map(|value| quote! { #value }).collect(),
        quote! { i64 },
    )
}

fn quantize_usage(usage: f32, scale: u32, preserve_nonzero: bool) -> Result<u32> {
    anyhow::ensure!(usage.is_finite() && usage >= 0.0, "invalid usage: {usage}");

    let quantized = (usage * scale as f32).round();
    anyhow::ensure!(quantized <= u32::MAX as f32, "usage is too large: {usage}");
    let quantized = quantized as u32;

    Ok(if preserve_nonzero && usage > 0.0 && quantized == 0 {
        1
    } else {
        quantized
    })
}

fn run_node(script: &str) -> Result<String> {
    use std::process::Command;
    let out = Command::new("node").arg("-e").arg(script).output()?;
    if !out.status.success() {
        anyhow::bail!("node failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(String::from_utf8(out.stdout)?)
}

#[derive(Default)]
struct StrPool {
    pool: String,
    map: HashMap<String, u32>,
}

impl StrPool {
    pub fn insert(&mut self, s: &str) -> u32 {
        if let Some(id) = self.map.get(s) {
            return *id;
        }

        let offset = self.pool.len();
        self.pool.push_str(s);
        let len: u8 = s.len().try_into().unwrap();
        let offset: u32 = offset.try_into().unwrap();

        if offset > (1 << 24) {
            panic!("string too large");
        }

        let id = offset | (u32::from(len) << 24);
        self.map.insert(s.to_owned(), id);
        id
    }

    pub fn get(&self, id: u32) -> &str {
        // 24bit offset and 8bit len
        let offset = id & ((1 << 24) - 1);
        let len = id >> 24;

        &self.pool[(offset as usize)..][..(len as usize)]
    }
}
