use super::{Distrib, QueryResult};

pub(super) fn firefox_esr() -> QueryResult {
    Ok(vec![
        Distrib::new("firefox", "128"),
        Distrib::new("firefox", "140"),
    ])
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("firefox esr"; "firefox")]
    #[test_case("Firefox ESR"; "firefox case insensitive")]
    #[test_case("ff esr"; "ff")]
    #[test_case("FF ESR"; "ff case insensitive")]
    #[test_case("fx esr"; "fx")]
    #[test_case("Fx ESR"; "fx case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
