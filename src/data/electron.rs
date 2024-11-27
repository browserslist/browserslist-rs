use crate::error::Error;
use nom::{
    character::complete::{char, u16},
    combinator::{all_consuming, opt},
    number::complete::float,
    sequence::{pair, terminated},
};
use std::sync::LazyLock;

pub static ELECTRON_VERSIONS: LazyLock<Vec<(f32, &'static str)>> =
    LazyLock::new(|| include!("../generated/electron-to-chromium.rs"));

pub(crate) fn parse_version(version: &str) -> Result<f32, Error> {
    all_consuming(terminated(float, opt(pair(char('.'), u16))))(version)
        .map(|(_, v)| v)
        .map_err(|_: nom::Err<nom::error::Error<_>>| {
            Error::UnknownElectronVersion(version.to_string())
        })
}
