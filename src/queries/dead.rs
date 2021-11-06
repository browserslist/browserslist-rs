use super::Selector;
use crate::{opts::Opts, resolve};

pub(super) struct DeadSelector;

impl Selector for DeadSelector {
    fn select(&self, text: &str, opts: &Opts) -> Option<Vec<String>> {
        if text.eq_ignore_ascii_case("dead") {
            Some(resolve(
                &[
                    "ie <= 10",
                    "ie_mob <= 11",
                    "bb <= 10",
                    "op_mob <= 12.1",
                    "samsung 4",
                ],
                opts,
            ))
        } else {
            None
        }
    }
}
