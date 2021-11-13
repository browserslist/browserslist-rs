use std::{cmp::Ordering, num::ParseIntError, str::FromStr};

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Debug, Clone)]
pub(crate) struct Version(u32, u32, u32);

impl FromStr for Version {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // this allows something like `4.4.3-4.4.4`
        let mut segments = s.split('-').next().unwrap().split('.');
        let major = match segments.next() {
            Some(n) => n.parse()?,
            None => 0,
        };
        let minor = match segments.next() {
            Some(n) => n.parse()?,
            None => 0,
        };
        let patch = match segments.next() {
            Some(n) => n.parse()?,
            None => 0,
        };

        Ok(Self(major, minor, patch))
    }
}

pub(crate) fn compare(a: &str, b: &str) -> Ordering {
    a.parse::<Version>()
        .unwrap_or_default()
        .cmp(&b.parse().unwrap_or_default())
}

pub(crate) fn loose_compare(a: &str, b: &str) -> Ordering {
    a.split('.')
        .take(2)
        .zip(b.split('.').take(2))
        .fold(Ordering::Equal, |ord, (a, b)| {
            if ord == Ordering::Equal {
                a.parse::<i32>()
                    .unwrap_or_default()
                    .cmp(&b.parse::<i32>().unwrap_or_default())
            } else {
                ord
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version() {
        assert_eq!(Ok(Version(1, 0, 0)), "1".parse());
        assert_eq!(Ok(Version(1, 2, 0)), "1.2".parse());
        assert_eq!(Ok(Version(1, 2, 3)), "1.2.3".parse());
        assert_eq!(Ok(Version(12, 34, 56)), "12.34.56".parse());

        assert_eq!(Ok(Version(1, 0, 0)), "1-2".parse());
        assert_eq!(Ok(Version(1, 2, 0)), "1.2-1.3".parse());
        assert_eq!(Ok(Version(1, 2, 3)), "1.2.3-1.2.4".parse());
        assert_eq!(Ok(Version(12, 34, 56)), "12.34.56-78.9".parse());

        assert!("tp".parse::<Version>().is_err());
    }
}
