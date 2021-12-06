use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    env, fs, io,
};

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
    stats: HashMap<String, HashMap<String, String>>,
}

fn main() -> Result<()> {
    #[cfg(feature = "node")]
    {
        napi_build::setup();
    }

    build_electron_to_chromium()?;
    build_node_versions()?;
    build_node_release_schedule()?;
    build_caniuse_global()?;
    build_caniuse_region()?;

    Ok(())
}

fn build_electron_to_chromium() -> Result<()> {
    println!("cargo:rerun-if-changed=vendor/electron-to-chromium/versions.json");

    let path = format!("{}/electron-to-chromium.json", env::var("OUT_DIR")?);

    let mut data = serde_json::from_slice::<BTreeMap<String, String>>(&fs::read(format!(
        "{}/vendor/electron-to-chromium/versions.json",
        env::var("CARGO_MANIFEST_DIR")?
    ))?)?
    .into_iter()
    .map(|(electron_version, chromium_version)| {
        (electron_version.parse::<f32>().unwrap(), chromium_version)
    })
    .collect::<Vec<_>>();
    data.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    fs::write(path, &serde_json::to_string(&data)?)?;

    Ok(())
}

fn build_node_versions() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        version: String,
    }

    println!("cargo:rerun-if-changed=vendor/node-releases/data/processed/envs.json");

    let path = format!("{}/node-versions.json", env::var("OUT_DIR")?);

    let releases: Vec<NodeRelease> = serde_json::from_slice(&fs::read(format!(
        "{}/vendor/node-releases/data/processed/envs.json",
        env::var("CARGO_MANIFEST_DIR")?
    ))?)?;

    fs::write(
        path,
        &serde_json::to_string(
            &releases
                .into_iter()
                .map(|release| release.version)
                .collect::<Vec<_>>(),
        )?,
    )?;

    Ok(())
}

fn build_node_release_schedule() -> Result<()> {
    println!(
        "cargo:rerun-if-changed=vendor/node-releases/data/release-schedule/release-schedule.json"
    );

    #[derive(Deserialize)]
    struct NodeRelease {
        start: String,
        end: String,
    }

    let path = format!("{}/node-release-schedule.json", env::var("OUT_DIR")?);

    let schedule: HashMap<String, NodeRelease> = serde_json::from_slice(&fs::read(format!(
        "{}/vendor/node-releases/data/release-schedule/release-schedule.json",
        env::var("CARGO_MANIFEST_DIR")?
    ))?)?;

    fs::write(
        path,
        &serde_json::to_string(
            &schedule
                .into_iter()
                .map(|(version, release)| {
                    (
                        version.trim_start_matches('v').to_owned(),
                        (release.start, release.end),
                    )
                })
                .collect::<HashMap<_, _>>(),
        )?,
    )?;

    Ok(())
}

fn build_caniuse_global() -> Result<()> {
    use itertools::Itertools;

    #[derive(Serialize)]
    struct BrowserStat {
        name: String,
        version_list: Vec<VersionDetail>,
    }

    let out_dir = env::var("OUT_DIR")?;

    let data = parse_caniuse_global()?;

    let browsers = data
        .agents
        .iter()
        .map(|(name, agent)| {
            (
                name,
                BrowserStat {
                    name: name.to_string(),
                    version_list: agent.version_list.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    fs::write(
        format!("{}/caniuse-browsers.json", &out_dir),
        &serde_json::to_string(&browsers)?,
    )?;

    let mut global_usage = data
        .agents
        .iter()
        .map(|(name, agent)| {
            agent
                .usage_global
                .iter()
                .map(|(version, usage)| (name.clone(), version.clone(), usage))
        })
        .flatten()
        .collect::<Vec<_>>();
    global_usage.sort_unstable_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap());
    fs::write(
        format!("{}/caniuse-global-usage.json", &out_dir),
        &serde_json::to_string(&global_usage)?,
    )?;

    let features_dir = format!("{}/features", &out_dir);
    if matches!(fs::File::open(&features_dir), Err(e) if e.kind() == io::ErrorKind::NotFound) {
        fs::create_dir(&features_dir)?;
    }
    for (name, feature) in &data.data {
        fs::write(
            format!("{}/{}.json", &features_dir, name),
            serde_json::to_string(
                &feature
                    .stats
                    .iter()
                    .map(|(name, versions)| {
                        versions
                            .iter()
                            .filter(|(_, stat)| stat.starts_with('y') || stat.starts_with('a'))
                            .map(|(version, _)| (name.clone(), version.clone()))
                    })
                    .flatten()
                    .collect::<Vec<_>>(),
            )?,
        )?;
    }
    let arms = data
        .data
        .keys()
        .map(|name| {
            format!(
                r#"    "{0}" => {{
        use once_cell::sync::Lazy;
        use serde_json::from_str;
        use ustr::Ustr;
        static STAT: Lazy<Vec<(Ustr, &'static str)>> = Lazy::new(|| {{
            from_str(include_str!(concat!(env!("OUT_DIR"), "/features/{0}.json"))).unwrap()
        }});
        Some(&*STAT)
    }},"#,
                name
            )
        })
        .join("\n");
    let caniuse_features_matching = format!("match name {{\n{}\n    _ => None\n}}", &arms);
    fs::write(
        format!("{}/caniuse-feature-matching.rs", &out_dir),
        caniuse_features_matching,
    )?;

    Ok(())
}

fn parse_caniuse_global() -> Result<Caniuse> {
    println!("cargo:rerun-if-changed=vendor/caniuse/fulldata-json/data-2.0.json");

    Ok(serde_json::from_slice(&fs::read(format!(
        "{}/vendor/caniuse/fulldata-json/data-2.0.json",
        env::var("CARGO_MANIFEST_DIR")?
    ))?)?)
}

fn build_caniuse_region() -> Result<()> {
    use itertools::Itertools;

    #[derive(Deserialize)]
    struct RegionData {
        data: HashMap<String, HashMap<String, Option<f32>>>,
    }

    let files = fs::read_dir(format!(
        "{}/vendor/caniuse/region-usage-json",
        env::var("CARGO_MANIFEST_DIR")?
    ))?
    .map(|entry| entry.map_err(anyhow::Error::from))
    .collect::<Result<Vec<_>>>()?;

    files.iter().for_each(|entry| {
        println!(
            "cargo:rerun-if-changed=vendor/caniuse/region-usage-json/{}",
            entry.file_name().into_string().unwrap()
        )
    });

    let out_dir = env::var("OUT_DIR")?;

    let Caniuse { agents, .. } = parse_caniuse_global()?;

    let region_dir = format!("{}/region", &out_dir);
    if matches!(fs::File::open(&region_dir), Err(e) if e.kind() == io::ErrorKind::NotFound) {
        fs::create_dir(&region_dir)?;
    }

    for file in &files {
        let RegionData { data } = serde_json::from_slice(&fs::read(file.path())?)?;
        let mut usage = data
            .into_iter()
            .map(|(name, stat)| {
                dbg!(&name);
                let agent = agents.get(&name).unwrap();
                stat.into_iter().filter_map(move |(version, usage)| {
                    let version = if version.as_str() == "0" {
                        agent.version_list.last().unwrap().version.clone()
                    } else {
                        version
                    };
                    usage.map(|usage| (name.clone(), version, usage))
                })
            })
            .flatten()
            .collect::<Vec<_>>();
        usage.sort_unstable_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap());
        fs::write(
            format!("{}/region/{}", &out_dir, file.file_name().to_str().unwrap()),
            serde_json::to_string(&usage)?,
        )?;
    }
    let arms = files
        .iter()
        .map(|file| {
            format!(
                r#"    "{0}" => {{
        static USAGE: Lazy<Vec<(Ustr, &'static str, f32)>> = Lazy::new(|| {{
            from_str(include_str!(concat!(env!("OUT_DIR"), "/region/{0}.json"))).unwrap()
        }});
        Some(&*USAGE)
    }},"#,
                file.path().file_stem().unwrap().to_str().unwrap()
            )
        })
        .join("\n");
    let caniuse_region_matching = format!(
        "{{
use once_cell::sync::Lazy;
use serde_json::from_str;
use ustr::Ustr;

match region {{
{}
    _ => None,
}}
}}",
        &arms
    );
    fs::write(
        format!("{}/caniuse-region-matching.rs", &out_dir),
        caniuse_region_matching,
    )?;

    Ok(())
}
