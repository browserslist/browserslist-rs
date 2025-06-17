use std::borrow::Borrow;

pub struct BinMap<'a, K, V>(pub(super) &'a [(K, V)]);

impl<K, V> BinMap<'_, K, V> {
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

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)>
        + ExactSizeIterator
        + DoubleEndedIterator
    {
        self.0.iter()
    }
}

// We define repr C instead of using tuple to ensure a stable memory layout.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(super) struct PairU32(pub U32, pub U32);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(super) struct U32(u32);

impl U32 {
    pub const fn get(self) -> u32 {
        self.0.to_le()
    }
}
