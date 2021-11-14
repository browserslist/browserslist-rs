use super::{query, Selector, SelectorResult};
use crate::{error::Error, opts::Opts};

pub(super) struct DeadSelector;

impl Selector for DeadSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if text.eq_ignore_ascii_case("dead") {
            [
                "ie <= 10",
                "ie_mob <= 11",
                "bb <= 10",
                "op_mob <= 12.1",
                "samsung 4",
            ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("dead"; "basic")]
    #[test_case("Dead"; "case insensitive")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case("> 0%, dead"; "all browsers")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, &Opts::new().mobile_to_desktop(true));
    }
}
