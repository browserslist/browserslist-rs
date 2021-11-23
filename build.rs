use anyhow::Result;
use std::{collections::BTreeMap, env, fs};

const E2C: &str = "1.3.906";

fn main() -> Result<()> {
    #[cfg(feature = "node")]
    {
        napi_build::setup();
    }

    println!("cargo:rerun-if-changed=build.rs");

    fetch_electron_to_chromium()?;

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
