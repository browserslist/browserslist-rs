use std::cmp::Ordering;

pub(crate) fn compare(a: &str, b: &str) -> Ordering {
    a.split('.')
        .zip(b.split('.'))
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

pub(crate) fn reverse_compare(a: &str, b: &str) -> Ordering {
    a.split('.')
        .zip(b.split('.'))
        .fold(Ordering::Equal, |ord, (a, b)| {
            if ord == Ordering::Equal {
                // this is intentional: version comes from high to low
                b.parse::<i32>()
                    .unwrap_or_default()
                    .cmp(&a.parse::<i32>().unwrap_or_default())
            } else {
                ord
            }
        })
}
