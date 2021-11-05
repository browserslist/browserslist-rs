use super::Selector;
use crate::resolve;

pub(super) struct DeadSelector;

impl Selector for DeadSelector {
    fn select(&self, text: &str) -> Option<Vec<String>> {
        if text.eq_ignore_ascii_case("dead") {
            Some(
                [
                    "ie <= 10",
                    "ie_mob <= 11",
                    "bb <= 10",
                    "op_mob <= 12.1",
                    "samsung 4",
                ]
                .map(|s| resolve(s))
                .concat()
                .to_vec(),
            )
        } else {
            None
        }
    }
}
