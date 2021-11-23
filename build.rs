use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
};

const E2C: &str = "1.3.906";
const NODE: &str = "2.0.1";
const CANIUSE: &str = "1.0.30001282";

fn main() -> Result<()> {
    #[cfg(feature = "node")]
    {
        napi_build::setup();
    }

    println!("cargo:rerun-if-changed=build.rs");

    fetch_electron_to_chromium()?;
    fetch_node_versions()?;
    fetch_node_release_schedule()?;
    fetch_caniuse_global()?;

    Ok(())
}

fn fetch_electron_to_chromium() -> Result<()> {
    let source = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/electron-to-chromium@{}/versions.js",
        E2C
    ))
    .call()?
    .into_string()?;

    let mut data = serde_json::from_str::<BTreeMap<String, String>>(
        source
            .trim_start_matches("module.exports = ")
            .trim_end_matches(';'),
    )?
    .into_iter()
    .map(|(electron_version, chromium_version)| {
        (electron_version.parse::<f32>().unwrap(), chromium_version)
    })
    .collect::<Vec<_>>();
    data.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    fs::write(
        format!("{}/electron-to-chromium.json", env::var("OUT_DIR")?),
        &serde_json::to_string(&data)?,
    )?;

    Ok(())
}

fn fetch_node_versions() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        version: String,
    }

    let releases = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/node-releases@{}/data/processed/envs.json",
        NODE
    ))
    .call()?
    .into_json::<Vec<NodeRelease>>()?;

    fs::write(
        format!("{}/node-versions.json", env::var("OUT_DIR")?),
        &serde_json::to_string(
            &releases
                .into_iter()
                .map(|release| release.version)
                .collect::<Vec<_>>(),
        )?,
    )?;

    Ok(())
}

fn fetch_node_release_schedule() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        start: String,
        end: String,
    }

    let schedule = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/node-releases@{}/data/release-schedule/release-schedule.json",
        NODE
    ))
    .call()?
    .into_json::<HashMap<String, NodeRelease>>()?;

    fs::write(
        format!("{}/node-release-schedule.json", env::var("OUT_DIR")?),
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

fn fetch_caniuse_global() -> Result<()> {
    #[derive(Deserialize)]
    struct Caniuse {
        agents: HashMap<String, Agent>,
    }

    #[derive(Deserialize)]
    struct Agent {
        usage_global: HashMap<String, f32>,
    }

    let data = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/caniuse-db@{}/fulldata-json/data-2.0.json",
        CANIUSE
    ))
    .call()?
    .into_json::<Caniuse>()?;

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
        format!("{}/caniuse-usage.json", env::var("OUT_DIR")?),
        &serde_json::to_string(&global_usage)?,
    )?;

    Ok(())
}
