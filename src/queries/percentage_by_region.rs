use super::{Distrib, QueryResult};
use crate::{data::caniuse::region::get_usage_by_region, error::Error, parser::Comparator};

pub(super) fn percentage_by_region(
    comparator: Comparator,
    popularity: f32,
    region: &str,
) -> QueryResult {
    let normalized_region = if region.len() == 2 {
        region.to_uppercase()
    } else {
        region.to_lowercase()
    };

    if let Some(region_data) = get_usage_by_region(&normalized_region) {
        let distribs = region_data
            .iter()
            .filter(|(_, _, usage)| match comparator {
                Comparator::Greater => *usage > popularity,
                Comparator::Less => *usage < popularity,
                Comparator::GreaterOrEqual => *usage >= popularity,
                Comparator::LessOrEqual => *usage <= popularity,
            })
            .map(|(name, version, _)| Distrib::new(name, *version))
            .collect();
        Ok(distribs)
    } else {
        Err(Error::UnknownRegion(region.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("> 10% in US"; "greater")]
    #[test_case(">= 5% in US"; "greater or equal")]
    #[test_case("< 5% in US"; "less")]
    #[test_case("<= 5% in US"; "less or equal")]
    #[test_case("> 10.2% in US"; "with float")]
    #[test_case("> .2% in US"; "with float that has a leading dot")]
    #[test_case("> 10.2% in us"; "fixes country case")]
    #[test_case("> 1% in RU"; "load country")]
    #[test_case("> 1% in alt-AS"; "load continents")]
    #[test_case(">10% in US"; "no space")]
    #[test_case("> 1% in CN"; "normalize incorrect caniuse versions for and-prefixed")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new(), None);
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("> 1% in XX", &Opts::new()),
            Error::UnknownRegion(String::from("XX"))
        );
    }
}
