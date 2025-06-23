use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1, take_while_m_n},
    character::complete::{anychar, char, i32, one_of, space0, space1, u16, u32},
    combinator::{all_consuming, consumed, map, opt, recognize, value, verify},
    multi::{many0, many_till},
    number::complete::{double, float},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

type PResult<'a, Output> = IResult<&'a str, Output>;

#[derive(Debug, Clone)]
pub enum QueryAtom<'a> {
    Last {
        count: u16,
        major: bool,
        name: Option<&'a str>,
    },
    Unreleased(Option<&'a str>),
    Years(f64),
    Since {
        year: i32,
        month: u32,
        day: u32,
    },
    Percentage {
        comparator: Comparator,
        popularity: f32,
        stats: Stats<'a>,
    },
    Cover {
        coverage: f32,
        stats: Stats<'a>,
    },
    Supports(&'a str, Option<SupportKind>),
    Electron(VersionRange<'a>),
    Node(VersionRange<'a>),
    Browser(&'a str, VersionRange<'a>),
    FirefoxESR,
    OperaMini,
    CurrentNode,
    MaintainedNode,
    Phantom(bool),
    BrowserslistConfig,
    Defaults,
    Dead,
    Extends(&'a str),
    Unknown(&'a str), // unnecessary, but for better error report
}

#[derive(Debug, Clone)]
pub enum Stats<'a> {
    Global,
    Region(&'a str),
}

#[derive(Debug, Clone)]
pub enum SupportKind {
    Fully,
    Partially,
}

fn parse_version_keyword(input: &str) -> PResult<&str> {
    terminated(tag_no_case("version"), opt(char('s')))(input)
}

fn parse_last(input: &str) -> PResult<QueryAtom> {
    map(
        tuple((
            terminated(tag_no_case("last"), space1),
            terminated(u16, space1),
            opt(terminated(
                verify(
                    take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
                    |s: &str| {
                        !s.eq_ignore_ascii_case("version")
                            && !s.eq_ignore_ascii_case("versions")
                            && !s.eq_ignore_ascii_case("major")
                    },
                ),
                space1,
            )),
            opt(terminated(tag_no_case("major"), space1)),
            parse_version_keyword,
        )),
        |(_, count, name, major, _)| {
            if matches!(name, Some(name) if name.eq_ignore_ascii_case("major")) && major.is_none() {
                QueryAtom::Last {
                    count,
                    major: true,
                    name: None,
                }
            } else {
                QueryAtom::Last {
                    count,
                    major: major.is_some(),
                    name,
                }
            }
        },
    )(input)
}

fn parse_unreleased(input: &str) -> PResult<QueryAtom> {
    map(
        delimited(
            terminated(tag_no_case("unreleased"), space1),
            opt(terminated(
                take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
                space1,
            )),
            parse_version_keyword,
        ),
        QueryAtom::Unreleased,
    )(input)
}

fn parse_years(input: &str) -> PResult<QueryAtom> {
    map(
        delimited(
            terminated(tag_no_case("last"), space1),
            terminated(double, space1),
            terminated(tag_no_case("year"), opt(char('s'))),
        ),
        QueryAtom::Years,
    )(input)
}

fn parse_since(input: &str) -> PResult<QueryAtom> {
    map(
        tuple((
            terminated(tag_no_case("since"), one_of(" \t")),
            i32,
            opt(preceded(char('-'), u32)),
            opt(preceded(char('-'), u32)),
        )),
        |(_, year, month, day)| QueryAtom::Since {
            year,
            month: month.unwrap_or(1),
            day: day.unwrap_or(1),
        },
    )(input)
}

#[derive(Debug, Clone)]
pub enum Comparator {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

fn parse_compare_operator(input: &str) -> PResult<Comparator> {
    map(
        tuple((alt((char('<'), char('>'))), opt(char('=')))),
        |(relation, equals)| match relation {
            '<' if equals.is_some() => Comparator::LessOrEqual,
            '<' => Comparator::Less,
            '>' if equals.is_some() => Comparator::GreaterOrEqual,
            _ => Comparator::Greater,
        },
    )(input)
}

fn parse_region(input: &str) -> PResult<Stats> {
    map(
        recognize(preceded(
            opt(tag_no_case("alt-")),
            take_while_m_n(2, 2, char::is_alphabetic),
        )),
        Stats::Region,
    )(input)
}

fn parse_percentage(input: &str) -> PResult<QueryAtom> {
    map(
        tuple((
            terminated(parse_compare_operator, space0),
            terminated(float, char('%')),
            opt(preceded(
                tuple((space1, tag_no_case("in"), space1)),
                parse_region,
            )),
        )),
        |(comparator, value, stats)| QueryAtom::Percentage {
            comparator,
            popularity: value,
            stats: stats.unwrap_or(Stats::Global),
        },
    )(input)
}

fn parse_cover(input: &str) -> PResult<QueryAtom> {
    map(
        tuple((
            preceded(
                terminated(tag_no_case("cover"), space1),
                terminated(float, char('%')),
            ),
            opt(preceded(
                tuple((space1, tag_no_case("in"), space1)),
                parse_region,
            )),
        )),
        |(value, stats)| QueryAtom::Cover {
            coverage: value,
            stats: stats.unwrap_or(Stats::Global),
        },
    )(input)
}

fn parse_supports(input: &str) -> PResult<QueryAtom> {
    map(
        separated_pair(
            opt(terminated(
                alt((
                    value(SupportKind::Fully, tag_no_case("fully")),
                    value(SupportKind::Partially, tag_no_case("partially")),
                )),
                space1,
            )),
            terminated(tag_no_case("supports"), space1),
            take_while1(|c: char| c.is_alphanumeric() || c == '-'),
        ),
        |(kind, name)| QueryAtom::Supports(name, kind),
    )(input)
}

#[derive(Debug, Clone)]
pub enum VersionRange<'a> {
    Bounded(&'a str, &'a str),
    Unbounded(Comparator, &'a str),
    Accurate(&'a str),
}

fn parse_version(input: &str) -> PResult<&str> {
    take_while1(|c: char| c.is_ascii_digit() || c == '.')(input)
}

fn parse_version_range(input: &str) -> PResult<VersionRange> {
    alt((
        map(
            preceded(
                space1,
                separated_pair(
                    parse_version,
                    delimited(space0, char('-'), space0),
                    parse_version,
                ),
            ),
            |(from, to)| VersionRange::Bounded(from, to),
        ),
        map(
            preceded(
                space0,
                separated_pair(parse_compare_operator, space0, parse_version),
            ),
            |(comparator, version)| VersionRange::Unbounded(comparator, version),
        ),
        map(preceded(space1, parse_version), VersionRange::Accurate),
    ))(input)
}

fn parse_electron(input: &str) -> PResult<QueryAtom> {
    map(
        preceded(tag_no_case("electron"), parse_version_range),
        QueryAtom::Electron,
    )(input)
}

fn parse_node(input: &str) -> PResult<QueryAtom> {
    map(
        preceded(tag_no_case("node"), parse_version_range),
        QueryAtom::Node,
    )(input)
}

fn parse_browser(input: &str) -> PResult<QueryAtom> {
    map(
        pair(
            take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
            alt((
                parse_version_range,
                map(preceded(space1, tag_no_case("tp")), VersionRange::Accurate),
            )),
        ),
        |(name, version)| QueryAtom::Browser(name, version),
    )(input)
}

fn parse_firefox_esr(input: &str) -> PResult<QueryAtom> {
    value(
        QueryAtom::FirefoxESR,
        tuple((
            alt((tag_no_case("firefox"), tag_no_case("fx"), tag_no_case("ff"))),
            space1,
            tag_no_case("esr"),
        )),
    )(input)
}

fn parse_opera_mini(input: &str) -> PResult<QueryAtom> {
    value(
        QueryAtom::OperaMini,
        tuple((
            alt((tag_no_case("operamini"), tag_no_case("op_mini"))),
            space1,
            tag_no_case("all"),
        )),
    )(input)
}

fn parse_current_node(input: &str) -> PResult<QueryAtom> {
    value(
        QueryAtom::CurrentNode,
        tuple((tag_no_case("current"), space1, tag_no_case("node"))),
    )(input)
}

fn parse_maintained_node(input: &str) -> PResult<QueryAtom> {
    value(
        QueryAtom::MaintainedNode,
        tuple((
            tag_no_case("maintained"),
            space1,
            tag_no_case("node"),
            space1,
            tag_no_case("versions"),
        )),
    )(input)
}

fn parse_phantom(input: &str) -> PResult<QueryAtom> {
    map(
        preceded(
            terminated(tag_no_case("phantomjs"), space1),
            alt((tag("1.9"), tag("2.1"))),
        ),
        |version| QueryAtom::Phantom(version == "2.1"),
    )(input)
}

fn parse_browserslist_config(input: &str) -> PResult<QueryAtom> {
    value(
        QueryAtom::BrowserslistConfig,
        tag_no_case("browserslist config"),
    )(input)
}

fn parse_defaults(input: &str) -> PResult<QueryAtom> {
    value(QueryAtom::Defaults, tag_no_case("defaults"))(input)
}

fn parse_dead(input: &str) -> PResult<QueryAtom> {
    value(QueryAtom::Dead, tag_no_case("dead"))(input)
}

fn parse_extends(input: &str) -> PResult<QueryAtom> {
    map(
        preceded(
            terminated(tag_no_case("extends"), space1),
            take_while1(|c: char| {
                c.is_alphanumeric() || c == '-' || c == '_' || c == '@' || c == '/' || c == '.'
            }),
        ),
        QueryAtom::Extends,
    )(input)
}

fn parse_unknown(input: &str) -> PResult<QueryAtom> {
    map(
        recognize(many_till(anychar, parse_composition_operator)),
        QueryAtom::Unknown,
    )(input)
}

fn parse_query_atom(input: &str) -> PResult<QueryAtom> {
    alt((
        parse_last,
        parse_unreleased,
        parse_years,
        parse_since,
        parse_percentage,
        parse_cover,
        parse_supports,
        parse_electron,
        parse_node,
        parse_firefox_esr,
        parse_opera_mini,
        parse_current_node,
        parse_maintained_node,
        parse_phantom,
        parse_browser,
        parse_browserslist_config,
        parse_defaults,
        parse_dead,
        parse_extends,
        parse_unknown,
    ))(input)
}

#[derive(Debug)]
pub(crate) struct SingleQuery<'a> {
    pub(crate) raw: &'a str,
    pub(crate) atom: QueryAtom<'a>,
    pub(crate) negated: bool,
    pub(crate) is_and: bool,
}

fn parse_and(input: &str) -> PResult<bool> {
    value(true, delimited(space1, tag_no_case("and"), space1))(input)
}

fn parse_or(input: &str) -> PResult<bool> {
    alt((
        value(false, delimited(space0, char(','), space0)),
        value(false, delimited(space1, tag_no_case("or"), space1)),
    ))(input)
}

fn parse_composition_operator(input: &str) -> PResult<bool> {
    alt((parse_and, parse_or))(input)
}

fn parse_single_query(input: &str) -> PResult<SingleQuery> {
    map(
        tuple((
            parse_composition_operator,
            consumed(pair(
                opt(terminated(tag_no_case("not"), space1)),
                parse_query_atom,
            )),
        )),
        |(is_and, (raw, (negated, atom)))| SingleQuery {
            raw,
            atom,
            negated: negated.is_some(),
            is_and,
        },
    )(input)
}

pub(crate) fn parse_browserslist_query(input: &str) -> PResult<Vec<SingleQuery>> {
    let input = input.trim();
    // `many0` doesn't allow empty input, so we detect it here
    if input.is_empty() {
        return Ok(("", vec![]));
    }

    map(
        all_consuming(tuple((
            consumed(pair(
                // this isn't allowed, but for better error report
                opt(terminated(tag_no_case("not"), space1)),
                parse_query_atom,
            )),
            many0(parse_single_query),
        ))),
        |((first_raw, (negated, first)), mut queries)| {
            queries.insert(
                0,
                SingleQuery {
                    raw: first_raw,
                    atom: first,
                    negated: negated.is_some(),
                    is_and: false,
                },
            );
            queries
        },
    )(input)
}

pub(crate) fn parse_electron_version(version: &str) -> Result<f32, crate::error::Error> {
    all_consuming(terminated(float, opt(pair(char('.'), u16))))(version)
        .map(|(_, v)| v)
        .map_err(|_: nom::Err<nom::error::Error<_>>| {
            crate::error::Error::UnknownElectronVersion(version.to_string())
        })
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case(""; "empty")]
    #[test_case("ie >= 6, ie <= 7"; "comma")]
    #[test_case("ie >= 6 and ie <= 7"; "and")]
    #[test_case("ie < 11 and not ie 7"; "and with not")]
    #[test_case("last 1 Baidu version and not <2%"; "with not and one-version browsers as and query")]
    #[test_case("ie >= 6 or ie <= 7"; "or")]
    #[test_case("ie < 11 or not ie 7"; "or with not")]
    #[test_case("last 2 versions and > 1%"; "swc issue 4871")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
