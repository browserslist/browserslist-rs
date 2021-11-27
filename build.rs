use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    env, fs, io,
};

const E2C: &str = "1.4.4";
const NODE: &str = "2.0.1";
const CANIUSE: &str = "1.0.30001283";

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
    let path = format!("{}/electron-to-chromium.json", env::var("OUT_DIR")?);

    if env::var("DOCS_RS").is_ok() {
        fs::write(path, "[]")?;
        return Ok(());
    }

    let mut data = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/electron-to-chromium@{}/versions.json",
        E2C
    ))
    .call()?
    .into_json::<BTreeMap<String, String>>()?
    .into_iter()
    .map(|(electron_version, chromium_version)| {
        (electron_version.parse::<f32>().unwrap(), chromium_version)
    })
    .collect::<Vec<_>>();
    data.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    fs::write(path, &serde_json::to_string(&data)?)?;

    Ok(())
}

fn fetch_node_versions() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        version: String,
    }

    let path = format!("{}/node-versions.json", env::var("OUT_DIR")?);

    if env::var("DOCS_RS").is_ok() {
        fs::write(path, "[]")?;
        return Ok(());
    }

    let releases = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/node-releases@{}/data/processed/envs.json",
        NODE
    ))
    .call()?
    .into_json::<Vec<NodeRelease>>()?;

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

fn fetch_node_release_schedule() -> Result<()> {
    #[derive(Deserialize)]
    struct NodeRelease {
        start: String,
        end: String,
    }

    let path = format!("{}/node-release-schedule.json", env::var("OUT_DIR")?);

    if env::var("DOCS_RS").is_ok() {
        fs::write(path, "{}")?;
        return Ok(());
    }

    let schedule = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/node-releases@{}/data/release-schedule/release-schedule.json",
        NODE
    ))
    .call()?
    .into_json::<HashMap<String, NodeRelease>>()?;

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

fn fetch_caniuse_global() -> Result<()> {
    use itertools::Itertools;

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

    #[derive(Serialize)]
    struct BrowserStat {
        name: String,
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

    let out_dir = env::var("OUT_DIR")?;

    if env::var("DOCS_RS").is_ok() {
        fs::write(format!("{}/caniuse-browsers.json", &out_dir), "{}")?;
        fs::write(format!("{}/caniuse-usage.json", &out_dir), "[]")?;
        fs::write(
            format!("{}/caniuse-feature-matching.rs", &out_dir),
            "match name { _ => unreachable!() }",
        )?;
        fs::write(format!("{}/caniuse-features-list.json", &out_dir), "[]")?;
        return Ok(());
    }

    let data = ureq::get(&format!(
        "https://cdn.jsdelivr.net/npm/caniuse-db@{}/fulldata-json/data-2.0.json",
        CANIUSE
    ))
    .call()?
    .into_json::<Caniuse>()?;

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
        format!("{}/caniuse-usage.json", &out_dir),
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
                r#"    "{0}" => include_str!(concat!(env!("OUT_DIR"), "/features/{0}.json")),"#,
                name
            )
        })
        .join("\n");
    let caniuse_features_matching =
        format!("match name {{\n{}\n    _ => unreachable!()\n}}", &arms);
    fs::write(
        format!("{}/caniuse-feature-matching.rs", &out_dir),
        caniuse_features_matching,
    )?;

    fs::write(
        format!("{}/caniuse-features-list.json", &out_dir),
        serde_json::to_string(&data.data.keys().collect::<Vec<_>>())?,
    )?;

    Ok(())
}
