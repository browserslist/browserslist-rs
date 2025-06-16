use std::borrow::Borrow;

pub(crate) mod caniuse;
pub(crate) mod electron;
pub(crate) mod node;

#[doc(hidden)]
pub fn decode_browser_name(id: u8) -> &'static str {
    match id {
        1 => "ie",
        2 => "edge",
        3 => "firefox",
        4 => "chrome",
        5 => "safari",
        6 => "opera",
        7 => "ios_saf",
        8 => "op_mini",
        9 => "android",
        10 => "bb",
        11 => "op_mob",
        12 => "and_chr",
        13 => "and_ff",
        14 => "ie_mob",
        15 => "and_uc",
        16 => "samsung",
        17 => "and_qq",
        18 => "baidu",
        19 => "kaios",
        _ => unreachable!("cannot recognize browser id"),
    }
}

pub struct BinaryMap<'a, K, V>(&'a [(K, V)]);

impl<K, V> BinaryMap<'_, K, V> {
    pub fn get<Q>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(q).map(|(_, v)| v)
    }

    pub fn get_key_value<Q>(&self, q: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let idx = self.0.binary_search_by(|(k, _)| k.borrow().cmp(&q)).ok()?;
        let item = &self.0[idx];
        Some((&item.0, &item.1))
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
    }
}
