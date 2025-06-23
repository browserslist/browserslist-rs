use std::ops::Range;

include!("generated/electron-to-chromium.rs");

pub fn versions() -> impl ExactSizeIterator<Item = (f32, &'static str)> + DoubleEndedIterator {
    ELECTRON_VERSIONS
        .iter()
        .copied()
        .zip(CHROMIUM_VERSIONS.iter().copied())
}

pub fn get(electron_version: f32) -> Option<&'static str> {
    let index = ELECTRON_VERSIONS
        .binary_search_by(|probe| probe.total_cmp(&electron_version))
        .ok()?;
    CHROMIUM_VERSIONS.get(index).copied()
}

pub fn bounded_range(range: Range<f32>) -> Result<&'static [&'static str], f32> {
    let start = ELECTRON_VERSIONS
        .binary_search_by(|probe| probe.total_cmp(&range.start))
        .map_err(|_| range.start)?;
    let end = ELECTRON_VERSIONS
        .binary_search_by(|probe| probe.total_cmp(&range.end))
        .map_err(|_| range.end)?;

    Ok(&CHROMIUM_VERSIONS[start..=end])
}
