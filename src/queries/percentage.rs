use super::{Distrib, QueryResult};
use crate::parser::Comparator;
use browserslist_data::caniuse;

pub(super) fn percentage(comparator: Comparator, popularity: f32) -> QueryResult {
    let distribs = caniuse::iter_browser_stat(false)
        .flat_map(|(name, list)| {
            list.iter()
                .filter(|version| {
                    let usage = version.global_usage;
                    match comparator {
                        Comparator::Greater => usage > popularity,
                        Comparator::GreaterOrEqual => usage >= popularity,
                        Comparator::Less => usage < popularity,
                        Comparator::LessOrEqual => usage <= popularity,
                    }
                })
                .map(move |version| Distrib::new(name, version.version()))
        })
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("> 10%"; "greater")]
    #[test_case(">= 5%"; "greater or equal")]
    #[test_case("< 5%"; "less")]
    #[test_case("<= 5%"; "less or equal")]
    #[test_case(">10%"; "no space")]
    #[test_case("> 10.2%"; "with float")]
    #[test_case("> .2%"; "with float that has a leading dot")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
