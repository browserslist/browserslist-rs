use std::borrow::Borrow;
use std::fmt;

pub(super) struct BinMap<'a, K, V>(pub(super) &'a [(K, V)]);

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
        let idx = self
            .0
            .binary_search_by(|(k, _)| Ord::cmp(k.borrow(), q))
            .ok()?;
        let item = &self.0[idx];
        Some((&item.0, &item.1))
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &(K, V)> + DoubleEndedIterator {
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

#[derive(Clone, Copy)]
pub(super) struct PooledStr(pub(super) u32);

impl PooledStr {
    pub fn as_str(&self) -> &'static str {
        static STRPOOL: &str = include_str!("generated/caniuse-strpool.bin");

        // 24bit offset and 8bit len
        let offset = self.0 & ((1 << 24) - 1);
        let len = self.0 >> 24;

        &STRPOOL[(offset as usize)..][..(len as usize)]
    }
}

impl Borrow<str> for PooledStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PooledStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for PooledStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
