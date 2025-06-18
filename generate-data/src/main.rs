use anyhow::Result;
use indexmap::IndexMap;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs, io::{self, Write},
};

const OUT_DIR: &str = "src/generated";

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
    agents: HashMap<String, Agent>,
    data: BTreeMap<String, Feature>,
}

#[derive(Deserialize)]
struct Agent {
    usage_global: HashMap<String, f32>,
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
    stats: HashMap<String, IndexMap<String, String>>,
}

fn main() -> Result<()> {
    build_electron_to_chromium()?;
    build_node_versions()?;
    build_node_release_schedule()?;
    build_caniuse_global()?;
    build_caniuse_region()?;

    Ok(())
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
        static ELECTRON_VERSIONS: &'static [f32] = &[ #(#electron_versions),* ];
        static CHROMIUM_VERSIONS: &'static [&'static str] = &[ #(#chromium_versions),* ];
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
            static NODE_VERSIONS: &'static [&'static str] = &[#(#versions),*];
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

    let schedule: HashMap<String, NodeRelease> = serde_json::from_slice(&fs::read(
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
            static NODE_RELEASE_VERSIONS: &'static [&'static str] = &[#(#versions),*];
            static NODE_RELEASE_SCHEDULE: &'static [(chrono::NaiveDate, chrono::NaiveDate)] = &[#(#dates),*];
        }
        .to_string(),
    )?;

    Ok(())
}

fn build_caniuse_global() -> Result<()> {
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
        let stats = stats
            .into_iter()
            .map(|(name_str_id, start, end)| {
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
                static VERSION_LIST: &'static [VersionDetail] = &[#(#versions),*];
                static BROWSERS_STATS: &'static [(PooledStr, BrowserStat)] = &[#(#stats),*];
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
        let push_usage = global_usage.into_iter().map(
            |(name_str_id, version_str_id, usage)| {
                quote! {
                    (
                        PooledStr(#name_str_id),
                        PooledStr(#version_str_id),
                        #usage
                    )
                }
            },
        );
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
                let mut list = ver.iter()
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

        let (stats_name, stats_list): (Vec<_>, Vec<_>) = stats.iter()
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
            versions.iter().copied()
        )?;
        let version_index_len = write_u32(
            format!("{OUT_DIR}/caniuse-feature-versionindex.u32seq"),
            stats_list.iter().flatten().copied()
        )?;

        fs::write(
            format!("{OUT_DIR}/caniuse-feature-flags.bin"),
            flags.as_slice()
        )?;
        fs::write(
            format!("{OUT_DIR}/caniuse-feature-browsers.bin"),
            stats_name.as_slice()
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
                    &core::mem::transmute(*include_bytes!("caniuse-feature-versionstore.u32seq"))
                };
                static FEATURES_STAT_VERSION_INDEX: &[PairU32; #version_index_len / core::mem::size_of::<PairU32>()] = unsafe {
                    &core::mem::transmute(*include_bytes!("caniuse-feature-versionindex.u32seq"))
                };
            }.to_string()
        )?;
    }

    fs::write(
        format!("{OUT_DIR}/caniuse-strpool.bin"),
        strpool.pool.as_bytes()
    )?;

    Ok(())
}

fn parse_caniuse_global() -> Result<Caniuse> {
    Ok(serde_json::from_slice(&fs::read(
        "vendor/caniuse/fulldata-json/data-2.0.json",
    )?)?)
}

fn build_caniuse_region() -> Result<()> {
    #[derive(Deserialize)]
    struct RegionData {
        data: HashMap<String, HashMap<String, Option<f32>>>,
    }

    let files = fs::read_dir("vendor/caniuse/region-usage-json")?
        .map(|entry| entry.map_err(anyhow::Error::from))
        .collect::<Result<Vec<_>>>()?;

    let Caniuse { agents, .. } = parse_caniuse_global()?;

    let region_dir = format!("{OUT_DIR}/region");
    if matches!(fs::File::open(&region_dir), Err(e) if e.kind() == io::ErrorKind::NotFound) {
        fs::create_dir(&region_dir)?;
    }

    for file in &files {
        let RegionData { data } = serde_json::from_slice(&fs::read(file.path())?)?;
        let mut usage = data
            .into_iter()
            .flat_map(|(name, stat)| {
                let agent = agents.get(&name).unwrap();
                stat.into_iter().filter_map(move |(version, usage)| {
                    let version = if version.as_str() == "0" {
                        agent.version_list.last().unwrap().version.clone()
                    } else {
                        version
                    };
                    usage.map(|usage| (encode_browser_name(&name), version, usage))
                })
            })
            .collect::<Vec<_>>();
        usage.sort_unstable_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap());
        fs::write(
            format!("{OUT_DIR}/region/{}", file.file_name().to_str().unwrap()),
            serde_json::to_string(&usage)?,
        )?;
    }
    let mut regions = files
        .iter()
        .map(|entry| {
            entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .map(|s| s.to_owned())
                .unwrap()
        })
        .collect::<Vec<_>>();
    regions.sort();
    let regions_len = regions.len();
    let tokens = quote! {{
        use serde_json::from_str;
        use std::sync::LazyLock;
        use crate::data::decode_browser_name;

        type Usage = LazyLock<Vec<(&'static str, &'static str, f32)>>;
        type Json = Vec<(u8, &'static str, f32)>;

        #[inline(never)]
        fn usage(data: &'static str) -> Vec<(&'static str, &'static str, f32)> {
            from_str::<Json>(data)
                .unwrap()
                .into_iter()
                .map(|(browser, version, usage)| (decode_browser_name(browser), version, usage))
                .collect()
        }

        static REGIONS: &[&str] = &[
            #( #regions ),*
        ];
        static USAGES: [Usage; #regions_len] = [
            #( LazyLock::new(|| usage(include_str!(concat!("region/", #regions, ".json")))) ),*
        ];

        let idx = REGIONS.binary_search(&region).ok()?;
        USAGES.get(idx).map(|v| &**v)
    }};
    fs::write(
        format!("{OUT_DIR}/caniuse-region-matching.rs"),
        tokens.to_string(),
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
    map: HashMap<&'s str, u32>
}

impl<'s> StrPool<'s> {
    pub fn insert(&mut self, s: &'s str) -> u32 {
        *self.map.entry(s).or_insert_with(|| {
            let offset = self.pool.len();
            self.pool.push_str(s);
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
