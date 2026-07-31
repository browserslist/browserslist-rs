use anyhow::Result;
use indexmap::IndexMap;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::{self, Write},
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

            let date = quote! {
                (
                    chrono::NaiveDate::from_ymd_opt(#start_year, #start_month, #start_day).unwrap(),
                    chrono::NaiveDate::from_ymd_opt(#end_year, #end_month, #end_day).unwrap(),
                )
            };

            (version.to_owned(), date)
        })
        .unzip();

    fs::write(
        path,
        quote! {
            static NODE_RELEASE_VERSIONS: &[&str] = &[#(#versions),*];
            static NODE_RELEASE_SCHEDULE: &[(chrono::NaiveDate, chrono::NaiveDate)] = &[#(#dates),*];
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
        let mut versions = Vec::new();
        let mut stats = Vec::new();

        for (name, agent) in &data.agents {
            let name_str_id = strpool.insert(name);
            let start: u32 = versions.len().try_into().unwrap();

            for version in &agent.version_list {
                let version_str_id = strpool.insert(&version.version);
                let usage = version.global_usage;
                let date = version.release_date.unwrap_or_default();
                let released = version.release_date.is_some();

                versions.push(quote! {
                    VersionDetail {
                        version: PooledStr(#version_str_id),
                        release_date: #date,
                        released: #released,
                        global_usage: #usage,
                    }
                });
            }

            let end: u32 = versions.len().try_into().unwrap();
            stats.push((name_str_id, start, end));
        }

        stats.sort_by_key(|(name_str_id, ..)| strpool.get(*name_str_id));
        let stats = stats.into_iter().map(|(name_str_id, start, end)| {
            quote! {
                (
                    PooledStr(#name_str_id),
                    BrowserStat(#start, #end)
                )
            }
        });

        fs::write(
            format!("{OUT_DIR}/caniuse-browsers.rs"),
            quote! {
                static VERSION_LIST: &[VersionDetail] = &[#(#versions),*];
                static BROWSERS_STATS: &[(PooledStr, BrowserStat)] = &[#(#stats),*];
            }
            .to_string(),
        )?;
    }

    // caniuse usage
    {
        let mut global_usage = Vec::new();
        for (name, agent) in &data.agents {
            let name_str_id = strpool.insert(name);
            for (version, usage) in &agent.usage_global {
                let version_str_id = strpool.insert(version);
                global_usage.push((name_str_id, version_str_id, usage));
            }
        }

        global_usage.sort_unstable_by(|(.., a), (.., b)| b.total_cmp(a));
        let push_usage = global_usage
            .into_iter()
            .map(|(name_str_id, version_str_id, usage)| {
                quote! {
                    (
                        PooledStr(#name_str_id),
                        PooledStr(#version_str_id),
                        #usage
                    )
                }
            });
        fs::write(
            format!("{OUT_DIR}/caniuse-global-usage.rs"),
            quote! {
                &[#(#push_usage),*]
            }
            .to_string(),
        )?;
    }

    // caniuse features
    {
        let mut features = Vec::new();
        let mut stats = Vec::new();
        let mut versions = Vec::new();
        let mut flags = Vec::new();

        for (name, feature) in &data.data {
            let start = stats.len();
            for (browser, ver) in &feature.stats {
                let mut list = ver
                    .iter()
                    .map(|(version, flags)| {
                        let version_str_id = strpool.insert(version);

                        let mut bit: u8 = 0;
                        if flags.contains('y') {
                            bit |= 1;
                        }
                        if flags.contains('a') {
                            bit |= 2;
                        }
                        (version_str_id, bit)
                    })
                    .collect::<Vec<_>>();

                // we only use `.get()`, so the original order does not need to be preserved here
                list.sort_by_key(|(x, _)| strpool.get(*x));

                let start = versions.len();
                versions.extend(list.iter().map(|(x, _)| *x));
                flags.extend(list.iter().map(|(_, y)| *y));
                let end = versions.len();

                stats.push((browser.as_str(), start, end));
            }
            let end = stats.len();

            stats[start..end].sort_by_key(|(browser, ..)| *browser);

            let name_str_id = strpool.insert(name);
            features.push((name_str_id, start, end));
        }

        features.sort_by_key(|(name, ..)| strpool.get(*name));

        let (stats_name, stats_list): (Vec<_>, Vec<_>) = stats
            .iter()
            .map(|(browser, start, end)| {
                let browser = encode_browser_name(browser);
                let start: u32 = (*start).try_into().unwrap();
                let end: u32 = (*end).try_into().unwrap();
                (browser, [start, end])
            })
            .unzip();
        let features = features.iter().flat_map(|(name_str_id, start, end)| {
            let start: u32 = (*start).try_into().unwrap();
            let end: u32 = (*end).try_into().unwrap();
            quote! {
                (
                    PooledStr(#name_str_id),
                    Feature(#start, #end)
                )
            }
        });

        let version_store_len = write_u32(
            format!("{OUT_DIR}/caniuse-feature-versionstore.u32seq"),
            versions.iter().copied(),
        )?;
        let version_index_len = write_u32(
            format!("{OUT_DIR}/caniuse-feature-versionindex.u32seq"),
            stats_list.iter().flatten().copied(),
        )?;

        fs::write(
            format!("{OUT_DIR}/caniuse-feature-flags.bin"),
            flags.as_slice(),
        )?;
        fs::write(
            format!("{OUT_DIR}/caniuse-feature-browsers.bin"),
            stats_name.as_slice(),
        )?;

        fs::write(
            format!("{OUT_DIR}/caniuse-feature-matching.rs"),
            quote! {
                static FEATURES: &[(PooledStr, Feature)] = &[#(#features),*];

                // # Safety
                //
                // We do the transmute at const context,
                // and the size and alignment are already checked and guaranteed by compiler.
                static FEATURES_STAT_VERSION_STORE: &[U32; #version_store_len / core::mem::size_of::<U32>()] = unsafe {
                    &core::mem::transmute::<
                        [u8; #version_store_len],
                        [U32; #version_store_len / core::mem::size_of::<U32>()]
                    >(*include_bytes!("caniuse-feature-versionstore.u32seq"))
                };
                static FEATURES_STAT_VERSION_INDEX: &[PairU32; #version_index_len / core::mem::size_of::<PairU32>()] = unsafe {
                    &core::mem::transmute::<
                        [u8; #version_index_len],
                        [PairU32; #version_index_len / core::mem::size_of::<PairU32>()]
                    >(*include_bytes!("caniuse-feature-versionindex.u32seq"))
                };

                static FEATURES_STAT_FLAGS: &[u8] = include_bytes!("caniuse-feature-flags.bin");
                static FEATURES_STAT_BROWSERS: &[u8] = include_bytes!("caniuse-feature-browsers.bin");
            }.to_string()
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
            usages[start..end].sort_by(|(_, _, a), (_, _, b)| b.total_cmp(a));

            let region_str_id = strpool.insert(region_name);
            region_usages.push((region_str_id, start, end));
        }

        region_usages.sort_by_key(|(region, ..)| strpool.get(*region));

        let browsers = usages.iter().map(|(b, ..)| *b).collect::<Vec<_>>();
        fs::write(format!("{OUT_DIR}/caniuse-region-browsers.bin"), &browsers)?;
        drop(browsers);

        let versions_len = write_u32(
            format!("{OUT_DIR}/caniuse-region-versions.u32seq"),
            usages.iter().map(|(_, v, _)| *v),
        )?;
        let usages_len = write_u32(
            format!("{OUT_DIR}/caniuse-region-usages.u32seq"),
            usages.iter().map(|(_, _, u)| u.to_bits()),
        )?;

        let region_data = region_usages
            .iter()
            .copied()
            .map(|(region_str_id, start, end)| {
                let start: u32 = start.try_into().unwrap();
                let end: u32 = end.try_into().unwrap();

                quote! {
                    (
                        PooledStr(#region_str_id),
                        RegionData(#start, #end)
                    )
                }
            });

        fs::write(
            format!("{OUT_DIR}/caniuse-region-matching.rs"),
            quote! {
                static REGIONS: &[(PooledStr, RegionData)] = &[#(#region_data),*];

                static REGIONS_BROWSERS: &[u8] = include_bytes!("caniuse-region-browsers.bin");
                static REGIONS_VERSIONS: &[U32; #versions_len / core::mem::size_of::<U32>()] = unsafe {
                    &core::mem::transmute::<
                        [u8; #versions_len],
                        [U32; #versions_len / core::mem::size_of::<U32>()]
                    >(*include_bytes!("caniuse-region-versions.u32seq"))
                };
                static REGIONS_USAGES: &[U32; #usages_len / core::mem::size_of::<U32>()] = unsafe {
                    &core::mem::transmute::<
                        [u8; #usages_len],
                        [U32; #usages_len / core::mem::size_of::<U32>()]
                    >(*include_bytes!("caniuse-region-usages.u32seq"))
                };
            }.to_string()
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
    let version_tokens = version_entries
        .iter()
        .map(|(browser, version)| quote! { (#browser, PooledStr(#version)) });
    let timeline_tokens = timeline_entries
        .iter()
        .map(|(date, start, end)| quote! { (#date, #start, #end) });
    fs::write(
        format!("{OUT_DIR}/baseline.rs"),
        quote! {
            static BASELINE_BROWSERS: &[u8] = &[#(#browser_tokens),*];
            static BASELINE_VERSIONS: &[(u8, PooledStr)] = &[#(#version_tokens),*];
            static BASELINE_TIMELINE: &[(u32, u16, u16)] = &[#(#timeline_tokens),*];
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

fn write_u32(path: String, iter: impl Iterator<Item = u32>) -> io::Result<usize> {
    let fd = fs::File::create(path)?;
    let mut fd = io::BufWriter::new(fd);
    let mut n = 0;

    for b in iter {
        fd.write_all(&b.to_le_bytes())?;
        n += 4;
    }

    fd.flush()?;
    Ok(n)
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
