use super::PooledStr;
use crate::{decode_browser_name, utils::BinMap};
use std::{iter, ops::Range, sync::LazyLock};

#[derive(Clone, Copy)]
pub struct Feature(u32, u32);

#[derive(Clone, Copy)]
pub struct VersionList {
    browser: u8,
    initial_flag: u8,
    last_flag: u8,
    positions: &'static [u16],
    flags: &'static [u8],
}

// ```rust
// static FEATURES_KEY/WIDTH: parallel feature name and browser-list widths
//
// static FEATURES_STAT_TRANSITION_HEADERS: &[u8]; // transition count << 4 | initial flag << 2 | last flag
// static FEATURES_STAT_TRANSITION_POSITIONS: &[u8]; // version position of each transition
// static FEATURES_STAT_FLAGS: &[u8]; // support flag from each transition on, except the last
// static FEATURES_STAT_BROWSERS: &[u8]; // browser name id
// ```
include!("../generated/caniuse-feature-matching.rs");

static FEATURES: LazyLock<Vec<(PooledStr, Feature)>> = LazyLock::new(|| {
    FEATURES_KEY
        .iter()
        .zip(&*FEATURES_WIDTH)
        .scan(0, |start, (key, width)| {
            let end = *start + u32::from(*width);
            let feature = (PooledStr(*key), Feature(*start, end));
            *start = end;
            Some(feature)
        })
        .collect()
});

static FEATURES_STAT_FLAG_START: LazyLock<Vec<(u32, u32)>> = LazyLock::new(|| {
    let (mut positions, mut flags) = (0, 0);
    FEATURES_STAT_TRANSITION_HEADERS
        .iter()
        .map(|header| {
            let base = (positions, flags);
            let count = u32::from(header >> 4);
            positions += count;
            flags += count.saturating_sub(1);
            base
        })
        .collect()
});

pub fn get_feature_stat(name: &str) -> Option<Feature> {
    BinMap(&FEATURES).get(name).copied()
}

fn version_list_at(index: usize) -> VersionList {
    let header = FEATURES_STAT_TRANSITION_HEADERS[index];
    let count = usize::from(header >> 4);
    let (positions, flags) = FEATURES_STAT_FLAG_START[index];
    let (positions, flags) = (positions as usize, flags as usize);
    VersionList {
        browser: FEATURES_STAT_BROWSERS[index],
        initial_flag: (header >> 2) & 3,
        last_flag: header & 3,
        positions: &FEATURES_STAT_TRANSITION_POSITIONS[positions..positions + count],
        flags: &FEATURES_STAT_FLAGS[flags..flags + count.saturating_sub(1)],
    }
}

impl Feature {
    pub fn get(&self, browser: &str) -> Option<VersionList> {
        let range = (self.0 as usize)..(self.1 as usize);
        let index = FEATURES_STAT_BROWSERS[range.clone()]
            .binary_search_by_key(&browser, |&k| decode_browser_name(k))
            .ok()?;
        Some(version_list_at(range.start + index))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, VersionList)> {
        let start = self.0 as usize;
        FEATURES_STAT_BROWSERS[start..(self.1 as usize)]
            .iter()
            .enumerate()
            .map(move |(offset, &id)| (decode_browser_name(id), version_list_at(start + offset)))
    }
}

impl VersionList {
    pub fn get(&self, version: &str) -> Option<u8> {
        let position = super::version_index(self.browser, version)?;
        Some(self.flag_at(position as usize))
    }

    /// Support flag of the version at `position` in this browser's version list.
    pub fn flag_at(&self, position: usize) -> u8 {
        let offset = self
            .positions
            .partition_point(|&transition| usize::from(transition) <= position);
        if offset == 0 {
            self.initial_flag
        } else if offset == self.positions.len() {
            self.last_flag
        } else {
            self.flags[offset - 1]
        }
    }

    /// Runs of adjacent positions in this browser's version list that share one support
    /// flag, in version list order, covering the first `version_count` positions.
    pub fn spans(&self, version_count: usize) -> impl Iterator<Item = (Range<usize>, u8)> + use<> {
        let positions = self.positions.iter().map(|&position| usize::from(position));
        let starts = iter::once(0).chain(positions.clone());
        let ends = positions.chain(iter::once(version_count));
        let flags = iter::once(self.initial_flag)
            .chain(self.flags.iter().copied())
            .chain((!self.positions.is_empty()).then_some(self.last_flag));
        starts
            .zip(ends)
            .zip(flags)
            .map(|((start, end), flag)| (start..end, flag))
    }
}
