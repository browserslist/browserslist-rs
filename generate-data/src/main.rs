use anyhow::Result;
use indexmap::IndexMap;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
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
    build_caniuse()?;
    build_baseline()?;

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
        "vendor/electron-to-chromium/versions.json",
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

    let releases: Vec<NodeRelease> =
        serde_json::from_slice(&fs::read("vendor/node-releases/data/processed/envs.json")?)?;

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
        "vendor/node-releases/data/release-schedule/release-schedule.json",
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

fn build_caniuse() -> Result<()> {
    let data = parse_caniuse_global()?;

    let mut strpool = StrPool::default();

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
        #[derive(Deserialize)]
        struct RegionData {
            data: BTreeMap<String, BTreeMap<String, Option<f32>>>,
        }

        let files = fs::read_dir("vendor/caniuse/region-usage-json")?
            .map(|entry| entry.map_err(anyhow::Error::from))
            .collect::<Result<Vec<_>>>()?;
        let mut usages = Vec::new();
        let mut region_usages = Vec::new();

        for file in &files {
            let RegionData { data: region_data } = serde_json::from_slice(&fs::read(file.path())?)?;

            let start = usages.len();
            for (name, stat) in &region_data {
                let agent = data.agents.get(name).unwrap();
                for (version, usage) in stat {
                    if let &Some(usage) = usage {
                        let version = if version.as_str() == "0" {
                            Cow::Borrowed(&*agent.version_list.last().unwrap().version)
                        } else {
                            Cow::Owned(version.clone())
                        };

                        let version_str_id = strpool.insert_cow(version);
                        usages.push((encode_browser_name(name), version_str_id, usage));
                    }
                }
            }
            let end = usages.len();
            usages[start..end].sort_by(|(_, _, a), (_, _, b)| b.total_cmp(a));

            let region_name = file
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let region_str_id = strpool.insert_cow(Cow::Owned(region_name));
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

    fs::write(
        format!("{OUT_DIR}/caniuse-strpool.bin"),
        strpool.pool.as_bytes(),
    )?;

    Ok(())
}

fn parse_caniuse_global() -> Result<Caniuse> {
    Ok(serde_json::from_slice(&fs::read(
        "vendor/caniuse/fulldata-json/data-2.0.json",
    )?)?)
}

fn run_node(script: &str) -> Result<String> {
    use std::process::Command;
    let out = Command::new("node").arg("-e").arg(script).output()?;
    if !out.status.success() {
        anyhow::bail!("node failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn build_baseline() -> Result<()> {
    let csv_content = fs::read_to_string(
        "vendor/baseline-browser-mapping/static/all_versions_with_supports.csv",
    )?;

    // Map CSV browser names to caniuse browser names
    let browser_map: HashMap<&str, &str> = [
        ("chrome", "chrome"),
        ("chrome_android", "and_chr"),
        ("edge", "edge"),
        ("firefox", "firefox"),
        ("firefox_android", "and_ff"),
        ("safari", "safari"),
        ("safari_ios", "ios_saf"),
    ]
    .into_iter()
    .collect();

    // Per year -> (caniuse_browser -> first_version), first insertion wins
    let mut year_mins: BTreeMap<u16, BTreeMap<&'static str, String>> = BTreeMap::new();
    // First version per browser with supports="widely"
    let mut widely_mins: BTreeMap<&'static str, String> = BTreeMap::new();
    // First version per browser with supports="newly"
    let mut newly_mins: BTreeMap<&'static str, String> = BTreeMap::new();

    for line in csv_content.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').map(|f| f.trim_matches('"')).collect();
        if fields.len() < 4 {
            continue;
        }
        let csv_browser = fields[0];
        let version = fields[1];
        let year_str = fields[2];
        let supports = fields[3];

        let Some(&caniuse_browser) = browser_map.get(csv_browser) else {
            continue;
        };

        if year_str != "pre_baseline" {
            if let Ok(year_num) = year_str.parse::<u16>() {
                year_mins
                    .entry(year_num)
                    .or_default()
                    .entry(caniuse_browser)
                    .or_insert_with(|| version.to_owned());
            }
        }

        if supports == "widely" {
            widely_mins
                .entry(caniuse_browser)
                .or_insert_with(|| version.to_owned());
        }

        if supports == "newly" {
            newly_mins
                .entry(caniuse_browser)
                .or_insert_with(|| version.to_owned());
        }
    }

    // baseline-widely.rs: &[(&str, &str)] sorted by browser name
    let widely_entries: Vec<_> = widely_mins
        .iter()
        .map(|(browser, version)| quote! { (#browser, #version) })
        .collect();
    fs::write(
        format!("{OUT_DIR}/baseline-widely.rs"),
        quote! { &[#(#widely_entries),*] }.to_string(),
    )?;

    // baseline-newly.rs: &[(&str, &str)] sorted by browser name
    let newly_entries: Vec<_> = newly_mins
        .iter()
        .map(|(browser, version)| quote! { (#browser, #version) })
        .collect();
    fs::write(
        format!("{OUT_DIR}/baseline-newly.rs"),
        quote! { &[#(#newly_entries),*] }.to_string(),
    )?;

    // baseline-years.rs: &[(u16, &str, &str)] sorted by (year, browser)
    let year_entries: Vec<_> = year_mins
        .iter()
        .flat_map(|(year, browsers)| {
            let year = *year;
            browsers
                .iter()
                .map(move |(browser, version)| quote! { (#year, #browser, #version) })
        })
        .collect();
    fs::write(
        format!("{OUT_DIR}/baseline-years.rs"),
        quote! { &[#(#year_entries),*] }.to_string(),
    )?;

    // baseline-features.rs: cumulative change points sorted by cutoff date
    // Each entry: (date, chrome, chrome_android, edge, firefox, firefox_android, safari, safari_ios)
    build_baseline_features()?;

    // baseline-downstream.rs: downstream browser engine-version -> browser-version mappings
    build_baseline_downstream()?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct FeatureChangePoint {
    date: String,
    c: String,
    ca: String,
    e: String,
    f: String,
    fa: String,
    s: String,
    si: String,
}

fn build_baseline_features() -> Result<()> {
    let json = run_node(r#"
const data = require('./vendor/baseline-browser-mapping/src/data/data.js').data;
const features = data.features;
const browsers = ['c','ca','e','f','fa','s','si'];
const cmp = (a, b) => {
    const [am=0,an=0]=a.split('.',2).map(Number);
    const [bm=0,bn=0]=b.split('.',2).map(Number);
    if(am!==bm)return am>bm?1:-1;
    if(an!==bn)return an>bn?1:-1;
    return 0;
};
const sorted = features.map(f=>({
    date:f[0].startsWith('≤')?f[0].slice(1):f[0],
    support:f[1]
})).sort((a,b)=>a.date.localeCompare(b.date));
let cur={c:'0',ca:'0',e:'0',f:'0',fa:'0',s:'0',si:'0'};
const pts=[];
let lastDate=null;
sorted.forEach(({date,support})=>{
    let changed=false;
    browsers.forEach(b=>{
        const v=support[b]||'0';
        if(cmp(v,cur[b])>0){cur[b]=v;changed=true;}
    });
    if(changed){
        if(date!==lastDate){pts.push({date,...cur});lastDate=date;}
        else{pts[pts.length-1]={date,...cur};}
    }
});
process.stdout.write(JSON.stringify(pts));
"#)?;

    let points: Vec<FeatureChangePoint> = serde_json::from_str(&json)?;

    let entries: Vec<_> = points
        .iter()
        .map(|p| {
            let (date, c, ca, e, f, fa, s, si) = (
                &p.date, &p.c, &p.ca, &p.e, &p.f, &p.fa, &p.s, &p.si,
            );
            quote! { (#date, #c, #ca, #e, #f, #fa, #s, #si) }
        })
        .collect();

    fs::write(
        format!("{OUT_DIR}/baseline-features.rs"),
        quote! {
            // (cutoff_date, chrome, chrome_android, edge, firefox, firefox_android, safari, safari_ios)
            // Sorted by date; each entry is the cumulative max min-versions as of that date.
            &[#(#entries),*]
        }
        .to_string(),
    )?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct DownstreamEntry {
    caniuse: String,
    engine: String,
    engine_version: u16,
    browser_version: String,
}

fn build_baseline_downstream() -> Result<()> {
    let json = run_node(r#"
const data = require('./vendor/baseline-browser-mapping/src/data/data.js').data;
const dsBrowsersFile = require('./vendor/baseline-browser-mapping/static/downstream-browsers.json');
const bbmToCaniuse = {
    webview_android:'android', samsunginternet_android:'samsung',
    opera_android:'op_mob', opera:'opera',
    qq_android:'and_qq', uc_android:'and_uc', kai_os:'kaios'
};
const result=[];
['webview_android','samsunginternet_android','opera_android','opera'].forEach(bname=>{
    const cname=bbmToCaniuse[bname];
    const b=data.bcdBrowsers[bname];
    if(!b)return;
    b.releases.filter(r=>r[3]==='b').forEach(r=>{
        result.push({caniuse:cname,engine:'blink',engine_version:parseInt(r[4]),browser_version:r[0]});
    });
});
['qq_android','uc_android'].forEach(bname=>{
    const cname=bbmToCaniuse[bname];
    const b=dsBrowsersFile.browsers[bname];
    if(!b)return;
    Object.entries(b.releases).filter(([,r])=>r.engine==='Blink').forEach(([v,r])=>{
        result.push({caniuse:cname,engine:'blink',engine_version:parseInt(r.engine_version),browser_version:v});
    });
});
const kaios=dsBrowsersFile.browsers['kai_os'];
if(kaios){
    Object.entries(kaios.releases).filter(([,r])=>r.engine==='Gecko').forEach(([v,r])=>{
        result.push({caniuse:'kaios',engine:'gecko',engine_version:parseInt(r.engine_version),browser_version:v});
    });
}
process.stdout.write(JSON.stringify(result));
"#)?;

    let entries: Vec<DownstreamEntry> = serde_json::from_str(&json)?;

    let blink: Vec<_> = entries
        .iter()
        .filter(|e| e.engine == "blink")
        .map(|e| {
            let (caniuse, ev, bv) = (&e.caniuse, e.engine_version, &e.browser_version);
            quote! { (#caniuse, #ev, #bv) }
        })
        .collect();

    let gecko: Vec<_> = entries
        .iter()
        .filter(|e| e.engine == "gecko")
        .map(|e| {
            let (caniuse, ev, bv) = (&e.caniuse, e.engine_version, &e.browser_version);
            quote! { (#caniuse, #ev, #bv) }
        })
        .collect();

    fs::write(
        format!("{OUT_DIR}/baseline-downstream.rs"),
        quote! {
            // (caniuse_browser, engine_major_version, browser_version)
            // For Blink-based downstream browsers (sorted by browser_version within each browser).
            static BASELINE_DOWNSTREAM_BLINK: &[(&str, u16, &str)] = &[#(#blink),*];
            // For Gecko-based downstream browsers.
            static BASELINE_DOWNSTREAM_GECKO: &[(&str, u16, &str)] = &[#(#gecko),*];
        }
        .to_string(),
    )?;

    Ok(())
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
struct StrPool<'s> {
    pool: String,
    map: HashMap<Cow<'s, str>, u32>,
}

impl<'s> StrPool<'s> {
    pub fn insert(&mut self, s: &'s str) -> u32 {
        self.insert_cow(Cow::Borrowed(s))
    }

    pub fn insert_cow(&mut self, s: Cow<'s, str>) -> u32 {
        *self.map.entry(s.clone()).or_insert_with(|| {
            let offset = self.pool.len();
            self.pool.push_str(&s);
            let len: u8 = (self.pool.len() - offset).try_into().unwrap();
            let offset: u32 = offset.try_into().unwrap();

            if offset > (1 << 24) {
                panic!("string too large");
            }

            offset | (u32::from(len) << 24)
        })
    }

    pub fn get(&self, id: u32) -> &str {
        // 24bit offset and 8bit len
        let offset = id & ((1 << 24) - 1);
        let len = id >> 24;

        &self.pool[(offset as usize)..][..(len as usize)]
    }
}
