use super::Selector;
use crate::{error::Error, opts::Opts, resolve};

pub(super) struct DeadSelector;

impl Selector for DeadSelector {
    fn select(&self, text: &str, opts: &Opts) -> Result<Option<Vec<String>>, Error> {
        if text.eq_ignore_ascii_case("dead") {
            resolve(
                &[
                    "ie <= 10",
                    "ie_mob <= 11",
                    "bb <= 10",
                    "op_mob <= 12.1",
                    "samsung 4",
                ],
                opts,
            )
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
