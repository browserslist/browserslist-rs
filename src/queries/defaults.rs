use super::{Selector, SelectorResult};
use crate::{opts::Opts, resolve};

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if text.eq_ignore_ascii_case("defaults") {
            resolve(
                ["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"],
                opts,
            )
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

    #[test_case("defaults", &Opts::new(); "no options")]
    #[test_case("Defaults", &Opts::new(); "case insensitive")]
    #[test_case("defaults", Opts::new().mobile_to_desktop(true); "respect options")]
    #[test_case("defaults, ie 6", &Opts::new(); "with other queries")]
    fn valid(query: &str, opts: &Opts) {
        run_compare(query, opts);
    }
}
