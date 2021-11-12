use super::{query, Selector, SelectorResult};
use crate::{error::Error, opts::Opts};

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if text.eq_ignore_ascii_case("defaults") {
            ["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"]
                .into_iter()
                .map(|q| query(q, opts))
                .try_fold(Vec::with_capacity(20), |mut result, current| {
                    result.append(&mut current?);
                    Ok::<_, Error>(result)
                })
                .map(Some)
        } else {
            Ok(None)
        }
    }
}
