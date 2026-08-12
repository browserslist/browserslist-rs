use anyhow::Result;
use indexmap::IndexMap;
use miniz_oxide::deflate::compress_to_vec;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::Write,
};

const OUT_DIR: &str = "data/src/generated";

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
    usage_global: BTreeMap<String, f32>,
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
    let mut versions_table = VersionTable::default();
    build_caniuse(&mut strpool, &mut versions_table)?;
    build_baseline(&mut strpool, &mut versions_table)?;
    fs::write(format!("{OUT_DIR}/strpool.bin"), strpool.pool.as_bytes())?;

    let versions = versions_table
        .ids
        .iter()
        .map(|id| quote! { PooledStr(#id) });
    fs::write(
        format!("{OUT_DIR}/version-table.rs"),
        quote! {
            static VERSION_TABLE: &[PooledStr] = &[#(#versions),*];
        }
        .to_string(),
    )?;

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
    let mut previous = 0;
    let electron_steps = electron_versions
        .into_iter()
        .map(|version| {
            let hundredths = hundredths(version)?;
            let step = u8::try_from(hundredths - previous)?;
            previous = hundredths;
            Ok(step)
        })
        .collect::<Result<Vec<_>>>()?;

    let code = quote! {
        static ELECTRON_VERSION_STEP: &[u8] = &[ #(#electron_steps),* ];
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

    let date_token = |date: chrono::NaiveDateTime| {
        let (year, month, day) = (date.year(), date.month(), date.day());
        quote! { chrono::NaiveDate::from_ymd_opt(#year, #month, #day).unwrap() }
    };
    let (versions, dates): (Vec<_>, Vec<_>) = versions
        .into_iter()
        .map(|(version, (start, end))| {
            let version = version.trim_start_matches('v');
            (version.to_owned(), (date_token(start), date_token(end)))
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

fn build_caniuse(strpool: &mut StrPool, versions_table: &mut VersionTable) -> Result<()> {
    let data = parse_caniuse_global()?;
    let region_data = parse_caniuse_regions()?;

    // caniuse browsers
    {
        let mut versions = Vec::new();
        let mut release_dates = Vec::new();
        let mut released_flags = Vec::new();
        let mut usages = Vec::new();
        let mut stats = Vec::new();
        let mut lex_order = Vec::new();

        for (name, agent) in &data.agents {
            let name_str_id = strpool.insert(name);
            let start: u32 = versions.len().try_into().unwrap();

            // Feature lookups take a version string, but the version list is in release
            // order; bundle the permutation that puts it in lexicographic order so that
            // the lookup can binary search through it.
            let mut order = (0..agent.version_list.len())
                .map(u8::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            order.sort_by_key(|position| &agent.version_list[usize::from(*position)].version);
            lex_order.extend(order);

            for version in &agent.version_list {
                let version_str_id = strpool.insert(&version.version);
                versions.push(versions_table.intern(version_str_id)?);
                release_dates.push(u32::try_from(version.release_date.unwrap_or_default())?);
                released_flags.push(version.release_date.is_some());
                usages.push(per_mille(version.global_usage)?);
            }

            let end: u32 = versions.len().try_into().unwrap();
            stats.push((name_str_id, start, end));
        }

        let lex_order = write_blob("caniuse-version-lex-order.bin", &lex_order)?;
        let (versions_lo, versions_hi) = byte_planes(&versions);
        let versions_lo = write_blob("caniuse-version-lo.bin", &versions_lo)?;
        let versions_hi = write_blob("caniuse-version-hi.bin", &versions_hi)?;
        let release_dates = zigzag_delta(release_dates.into_iter());

        stats.sort_by_key(|(name_str_id, ..)| strpool.get(*name_str_id));
        let stat_keys = stats
            .iter()
            .map(|(name_str_id, ..)| quote! { PooledStr(#name_str_id) });
        // The ranges are contiguous, so only their widths are stored; the starts are
        // rebuilt by prefix sum on first use.
        let stat_widths = contiguous_widths_u8(stats.iter().map(|(_, start, end)| (*start, *end)))?;

        fs::write(
            format!("{OUT_DIR}/caniuse-browsers.rs"),
            quote! {
                static VERSION_LIST_LEX_ORDER: Blob = #lex_order;
                static VERSION_LIST_VERSION_LO: Blob = #versions_lo;
                static VERSION_LIST_VERSION_HI: Blob = #versions_hi;
                static VERSION_LIST_RELEASE_DATE_DELTA: &[u32] = &[#(#release_dates),*];
                static VERSION_LIST_RELEASED: &[bool] = &[#(#released_flags),*];
                static VERSION_LIST_GLOBAL_USAGE: &[u16] = &[#(#usages),*];
                static BROWSERS_STATS_KEY: &[PooledStr] = &[#(#stat_keys),*];
                static BROWSERS_STATS_WIDTH: &[u8] = &[#(#stat_widths),*];
            }
            .to_string(),
        )?;
    }

    // caniuse usage
    {
        let mut global_usage = Vec::new();
        for (name, agent) in &data.agents {
            let browser = encode_browser_name(name);
            for (version, usage) in &agent.usage_global {
                let version_str_id = strpool.insert(version);
                global_usage.push((browser, versions_table.intern(version_str_id)?, usage));
            }
        }

        global_usage.sort_unstable_by(|(.., a), (.., b)| b.total_cmp(a));
        let browsers = write_blob(
            "caniuse-global-usage-browser.bin",
            &global_usage.iter().map(|(b, ..)| *b).collect::<Vec<_>>(),
        )?;
        let (versions_lo, versions_hi) =
            byte_planes(&global_usage.iter().map(|(_, v, _)| *v).collect::<Vec<_>>());
        let versions_lo = write_blob("caniuse-global-usage-version-lo.bin", &versions_lo)?;
        let versions_hi = write_blob("caniuse-global-usage-version-hi.bin", &versions_hi)?;
        let usages = global_usage
            .iter()
            .map(|(.., usage)| per_mille(**usage))
            .collect::<Result<Vec<_>>>()?;
        fs::write(
            format!("{OUT_DIR}/caniuse-global-usage.rs"),
            quote! {
                static CANIUSE_GLOBAL_USAGE_BROWSER: Blob = #browsers;
                static CANIUSE_GLOBAL_USAGE_VERSION_LO: Blob = #versions_lo;
                static CANIUSE_GLOBAL_USAGE_VERSION_HI: Blob = #versions_hi;
                static CANIUSE_GLOBAL_USAGE_USAGE: &[u16] = &[#(#usages),*];
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

                let start = flags.len();
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
                stats.push((browser.as_str(), start, flags.len()));
            }
            let end = stats.len();

            stats[start..end].sort_by_key(|(browser, ..)| *browser);

            let name_str_id = strpool.insert(name);
            features.push((name_str_id, start, end));
        }

        features.sort_by_key(|(name, ..)| strpool.get(*name));

        let stats_name = stats
            .iter()
            .map(|(browser, ..)| encode_browser_name(browser))
            .collect::<Vec<_>>();
        let feature_keys = zigzag_delta(features.iter().map(|(name_str_id, ..)| *name_str_id));
        let feature_widths = write_blob(
            "caniuse-feature-width.bin",
            &contiguous_widths_u8(
                features
                    .iter()
                    .map(|(_, start, end)| (*start as u32, *end as u32)),
            )?,
        )?;

        let version_widths = write_blob(
            "caniuse-feature-version-width.bin",
            &contiguous_widths_u8(
                stats
                    .iter()
                    .map(|(_, start, end)| (*start as u32, *end as u32)),
            )?,
        )?;

        // Two bits per flag; only `y` and `a` are recorded.
        let packed_flags = flags
            .chunks(4)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .fold(0u8, |byte, (i, flag)| byte | (flag << (2 * i)))
            })
            .collect::<Vec<_>>();
        let packed_flags = write_blob("caniuse-feature-flags.bin", &packed_flags)?;
        let stats_name = write_blob("caniuse-feature-browsers.bin", &stats_name)?;

        fs::write(
            format!("{OUT_DIR}/caniuse-feature-matching.rs"),
            quote! {
                static FEATURES_KEY_DELTA: &[u32] = &[#(#feature_keys),*];
                static FEATURES_WIDTH: Blob = #feature_widths;

                static FEATURES_STAT_VERSION_WIDTH: Blob = #version_widths;

                static FEATURES_STAT_FLAGS: Blob = #packed_flags;
                static FEATURES_STAT_BROWSERS: Blob = #stats_name;
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
            // Canonical order, identical in every region, so the browser and version
            // columns repeat and compress; `cover_by_region` re-sorts by usage, which is
            // the only query that depends on the order.
            usages[start..end].sort_by_key(|(browser, version, _)| (*browser, *version));

            let region_str_id = strpool.insert(region_name);
            region_usages.push((region_str_id, start, end));
        }

        region_usages.sort_by_key(|(region, ..)| strpool.get(*region));

        let browsers = write_blob(
            "caniuse-region-browsers.bin",
            &usages.iter().map(|(b, ..)| *b).collect::<Vec<_>>(),
        )?;

        let region_versions = usages
            .iter()
            .map(|(_, id, _)| versions_table.intern(*id))
            .collect::<Result<Vec<_>>>()?;
        let (region_versions_lo, region_versions_hi) = byte_planes(&region_versions);
        let region_versions_lo = write_blob("caniuse-region-version-lo.bin", &region_versions_lo)?;
        let region_versions_hi = write_blob("caniuse-region-version-hi.bin", &region_versions_hi)?;
        let region_percents = u32_planes(
            &usages
                .iter()
                .map(|(.., usage)| per_100k(*usage))
                .collect::<Result<Vec<_>>>()?,
        );
        let [usage_0, usage_1, usage_2, usage_3] = region_percents
            .iter()
            .enumerate()
            .map(|(i, plane)| write_blob(&format!("caniuse-region-usage-{i}.bin"), plane))
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected four usage planes"))?;

        let region_keys = zigzag_delta(
            region_usages
                .iter()
                .map(|(region_str_id, ..)| *region_str_id),
        );
        let region_widths = contiguous_widths(
            region_usages
                .iter()
                .map(|(_, start, end)| (*start as u32, *end as u32)),
        )?
        .into_iter()
        .map(|width| Ok(u16::try_from(width)?))
        .collect::<Result<Vec<_>>>()?;

        fs::write(
            format!("{OUT_DIR}/caniuse-region-matching.rs"),
            quote! {
                static REGIONS_KEY_DELTA: &[u32] = &[#(#region_keys),*];
                static REGIONS_WIDTH: &[u16] = &[#(#region_widths),*];

                static REGIONS_BROWSERS: Blob = #browsers;
                static REGIONS_VERSION_LO: Blob = #region_versions_lo;
                static REGIONS_VERSION_HI: Blob = #region_versions_hi;
                static REGIONS_USAGE_0: Blob = #usage_0;
                static REGIONS_USAGE_1: Blob = #usage_1;
                static REGIONS_USAGE_2: Blob = #usage_2;
                static REGIONS_USAGE_3: Blob = #usage_3;
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
        usage_global: agent.usage_global,
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

fn build_baseline(strpool: &mut StrPool, versions_table: &mut VersionTable) -> Result<()> {
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
    let (version_tokens_lo, version_tokens_hi) = byte_planes(
        &version_entries
            .iter()
            .map(|(_, version)| versions_table.intern(*version))
            .collect::<Result<Vec<_>>>()?,
    );
    let version_tokens_lo = write_blob("baseline-versions-version-lo.bin", &version_tokens_lo)?;
    let version_tokens_hi = write_blob("baseline-versions-version-hi.bin", &version_tokens_hi)?;
    let version_browser_tokens = write_blob(
        "baseline-versions-browser.bin",
        &version_entries
            .iter()
            .map(|(browser, _)| *browser)
            .collect::<Vec<_>>(),
    )?;
    let timeline_dates = zigzag_delta(timeline_entries.iter().map(|(date, ..)| *date));
    let timeline_widths = contiguous_widths_u8(
        timeline_entries
            .iter()
            .map(|(_, start, end)| (u32::from(*start), u32::from(*end))),
    )?;
    fs::write(
        format!("{OUT_DIR}/baseline.rs"),
        quote! {
            static BASELINE_BROWSERS: &[u8] = &[#(#browser_tokens),*];
            static BASELINE_VERSIONS_BROWSER: Blob = #version_browser_tokens;
            static BASELINE_VERSIONS_VERSION_LO: Blob = #version_tokens_lo;
            static BASELINE_VERSIONS_VERSION_HI: Blob = #version_tokens_hi;
            static BASELINE_TIMELINE_DATE_DELTA: &[u32] = &[#(#timeline_dates),*];
            static BASELINE_TIMELINE_WIDTH: &[u8] = &[#(#timeline_widths),*];
        }
        .to_string(),
    )?;

    Ok(())
}

fn run_node(script: &str) -> Result<String> {
    use std::process::Command;
    let out = Command::new("node").arg("-e").arg(script).output()?;
    if !out.status.success() {
        anyhow::bail!("node failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(String::from_utf8(out.stdout)?)
}

/// Checks that the ranges tile the array end to end and returns just their widths;
/// the starts are a prefix sum of these, rebuilt at load time.
fn contiguous_widths(ranges: impl Iterator<Item = (u32, u32)>) -> Result<Vec<u32>> {
    let mut widths = Vec::new();
    let mut expected = 0;
    for (start, end) in ranges {
        anyhow::ensure!(start == expected, "ranges must be contiguous");
        widths.push(end - start);
        expected = end;
    }
    Ok(widths)
}

/// [`contiguous_widths`], for ranges narrow enough to hold a width in a byte.
fn contiguous_widths_u8(ranges: impl Iterator<Item = (u32, u32)>) -> Result<Vec<u8>> {
    contiguous_widths(ranges)?
        .into_iter()
        .map(|width| Ok(u8::try_from(width)?))
        .collect()
}

/// Writes a byte array twice -- verbatim as `<name>`, and deflated as `<name>.deflate`
/// with its inflated length prefixed -- and returns the declaration of a `Blob` static
/// reading whichever one the `deflate` feature selects. Shipping both keeps the feature
/// a plain compile-time switch, with no compressor in anyone's build graph.
fn write_blob(name: &str, bytes: &[u8]) -> Result<TokenStream> {
    fs::write(format!("{OUT_DIR}/{name}"), bytes)?;

    let mut deflated = (bytes.len() as u32).to_le_bytes().to_vec();
    deflated.extend(compress_to_vec(bytes, 10));
    fs::write(format!("{OUT_DIR}/{name}.deflate"), &deflated)?;

    let deflate_name = format!("{name}.deflate");
    Ok(quote! {
        {
            #[cfg(feature = "deflate")]
            const BYTES: &[u8] = include_bytes!(#deflate_name);
            #[cfg(not(feature = "deflate"))]
            const BYTES: &[u8] = include_bytes!(#name);
            Blob::new(BYTES)
        }
    })
}

/// Splits a `u32` column into four byte arrays, one per byte position.
fn u32_planes(values: &[u32]) -> [Vec<u8>; 4] {
    let mut planes = [const { Vec::new() }; 4];
    for value in values {
        for (plane, byte) in planes.iter_mut().zip(value.to_le_bytes()) {
            plane.push(byte);
        }
    }
    planes
}

/// Splits a `u16` column into its low and high bytes, each kept contiguously. Every
/// index here points into a table of a few hundred versions, so the high plane is
/// almost entirely zero and all but vanishes once the data is compressed.
fn byte_planes(values: &[u16]) -> (Vec<u8>, Vec<u8>) {
    values
        .iter()
        .map(|value| ((value & 0xff) as u8, (value >> 8) as u8))
        .unzip()
}

/// One table of version strings, shared by every array that refers to a version so
/// that each reference costs a `u16` index rather than a four byte string id.
#[derive(Default)]
struct VersionTable {
    ids: Vec<u32>,
    index: HashMap<u32, u16>,
}

impl VersionTable {
    fn intern(&mut self, id: u32) -> Result<u16> {
        if let Some(index) = self.index.get(&id) {
            return Ok(*index);
        }
        let index = u16::try_from(self.ids.len())?;
        self.index.insert(id, index);
        self.ids.push(id);
        Ok(index)
    }
}

/// Successive differences, zigzagged so they stay unsigned. Values that drift by a
/// little compress far better this way; the array is summed back at load time.
fn zigzag_delta(values: impl Iterator<Item = u32>) -> Vec<u32> {
    let mut previous = 0i64;
    values
        .map(|value| {
            let delta = i64::from(value) - previous;
            previous = i64::from(value);
            ((delta << 1) ^ (delta >> 63)) as u32
        })
        .collect()
}

/// A usage percentage as thousandths. Global usage tops out around 45%, so a `u16`
/// holds it, and 0.001 is fine enough that no threshold query lands differently --
/// checked by the comparison tests against the JS implementation, which do fail at
/// hundredths.
fn per_mille(value: f32) -> Result<u16> {
    let scaled = (value * 1000.0).round();
    anyhow::ensure!(
        (0.0..65536.0).contains(&scaled),
        "usage out of range: {value}"
    );
    // `> 0%` is a real query, so a usage that is small but not zero must not round down
    // to zero; give it the smallest value that still counts as used.
    if value > 0.0 && scaled == 0.0 {
        return Ok(1);
    }
    Ok(scaled as u16)
}

/// A region usage percentage as hundred-thousandths. Region usage reaches 83%, which
/// overflows a `u16` at any scale finer than hundredths, and the caniuse region data
/// carries five decimals -- so this one stays 32 bits wide.
fn per_100k(value: f32) -> Result<u32> {
    let scaled = (value * 100_000.0).round();
    anyhow::ensure!(
        (0.0..4294967296.0).contains(&scaled),
        "usage out of range: {value}"
    );
    Ok(scaled as u32)
}

/// An electron version as hundredths, which represents every released version exactly.
fn hundredths(value: f32) -> Result<u16> {
    let scaled = (value * 100.0).round();
    anyhow::ensure!(
        (0.0..65536.0).contains(&scaled),
        "version out of range: {value}"
    );
    Ok(scaled as u16)
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
