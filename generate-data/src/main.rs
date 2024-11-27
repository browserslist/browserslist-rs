use anyhow::Result;
use indexmap::IndexMap;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
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
    data.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let data = data
        .into_iter()
        .map(|(electron_version, chromium_version)| {
            quote! {
                (#electron_version, #chromium_version)
            }
        });

    fs::write(
        path,
        quote! {{
            vec![#(#data),*]
        }}
        .to_string(),
    )?;

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
        quote! {{
            vec![#(#versions),*]
        }}
        .to_string(),
    )?;

    Ok(())
}

fn build_node_release_schedule() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        start: String,
        end: String,
    }

    let path = format!("{OUT_DIR}/node-release-schedule.rs");

    let schedule: HashMap<String, NodeRelease> = serde_json::from_slice(&fs::read(
        "vendor/node-releases/data/release-schedule/release-schedule.json",
    )?)?;
    let cap = schedule.len();
    let versions = schedule
        .into_iter()
        .map(|(version, NodeRelease { start, end })| {
            let version = version.trim_start_matches('v');
            quote! {
                map.insert(#version, (#start, #end));
            }
        });

    fs::write(
        path,
        quote! {{
            let mut map = ahash::AHashMap::with_capacity(#cap);
            #(#versions)*
            map
        }}
        .to_string(),
    )?;

    Ok(())
}

fn build_caniuse_global() -> Result<()> {
    let data = parse_caniuse_global()?;

    fs::write(
        format!("{OUT_DIR}/caniuse-browsers.rs"),
        {
            let map_cap = data.agents.len();
            let browser_stat = data.agents.iter().map(|(name, agent)| {
                let detail = agent.version_list.iter().map(|version| {
                    let ver = &version.version;
                    let global_usage = version.global_usage;
                    let release_date = if let Some(release_date) = version.release_date {
                        quote! { Some(#release_date) }
                    } else {
                        quote! { None }
                    };
                    quote! {
                        VersionDetail {
                            version: #ver,
                            global_usage: #global_usage,
                            release_date: #release_date,
                        }
                    }
                });
                quote! {
                    map.insert(#name, BrowserStat {
                        name: #name,
                        version_list: vec![#(#detail),*],
                    });
                }
            });
            quote! {{
                let mut map = AHashMap::with_capacity(#map_cap);
                #(#browser_stat)*
                map
            }}
        }
        .to_string(),
    )?;

    let mut global_usage = data
        .agents
        .iter()
        .flat_map(|(name, agent)| {
            agent.usage_global.iter().map(move |(version, usage)| {
                (
                    usage,
                    quote! {
                        (#name, #version, #usage)
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    global_usage.sort_unstable_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
    let push_usage = global_usage.into_iter().map(|(_, tokens)| tokens);
    fs::write(
        format!("{OUT_DIR}/caniuse-global-usage.rs"),
        quote! {
            vec![#(#push_usage),*]
        }
        .to_string(),
    )?;

    let features_dir = format!("{OUT_DIR}/features");
    if matches!(fs::File::open(&features_dir), Err(e) if e.kind() == io::ErrorKind::NotFound) {
        fs::create_dir(&features_dir)?;
    }
    for (name, feature) in &data.data {
        fs::write(
            format!("{features_dir}/{name}.json"),
            serde_json::to_string(
                &feature
                    .stats
                    .iter()
                    .map(|(name, versions)| {
                        (
                            encode_browser_name(name),
                            versions
                                .into_iter()
                                .map(|(version, flags)| {
                                    let mut bit = 0;
                                    if flags.contains('y') {
                                        bit |= 1;
                                    }
                                    if flags.contains('a') {
                                        bit |= 2;
                                    }
                                    (version, bit)
                                })
                                .collect::<IndexMap<_, u8>>(),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            )?,
        )?;
    }
    let features = data.data.keys().collect::<Vec<_>>();
    let tokens = quote! {{
        use ahash::AHashMap;
        use indexmap::IndexMap;
        use once_cell::sync::Lazy;
        use serde_json::from_str;
        use crate::data::decode_browser_name;

        type Stat = Lazy<AHashMap<&'static str, IndexMap<&'static str, u8>>>;
        type Json = AHashMap::<u8, IndexMap<&'static str, u8>>;

        match name {
            #( #features => {
                static STAT: Stat = Lazy::new(|| {
                    from_str::<Json>(include_str!(concat!("features/", #features, ".json")))
                        .unwrap()
                        .into_iter()
                        .map(|(browser, versions)| (decode_browser_name(browser), versions))
                        .collect()
                });
                Some(&*STAT)
            }, )*
            _ => None,
        }
    }};
    fs::write(
        format!("{OUT_DIR}/caniuse-feature-matching.rs"),
        tokens.to_string(),
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
    let regions = files
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
    let tokens = quote! {{
        use once_cell::sync::Lazy;
        use serde_json::from_str;
        use crate::data::decode_browser_name;

        type Usage = Lazy<Vec<(&'static str, &'static str, f32)>>;
        type Json = Vec<(u8, &'static str, f32)>;

        match region {
            #( #regions => {
                static USAGE: Usage = Lazy::new(|| {
                    from_str::<Json>(include_str!(concat!("region/", #regions, ".json")))
                        .unwrap()
                        .into_iter()
                        .map(|(browser, version, usage)| (decode_browser_name(browser), version, usage))
                        .collect()
                });
                Some(&*USAGE)
            }, )*
            _ => None,
        }
    }};
    fs::write(
        format!("{OUT_DIR}/caniuse-region-matching.rs"),
        tokens.to_string(),
    )?;

    Ok(())
}
