//! Bundled byte arrays, stored either verbatim or deflated.
//!
//! The selected representation is fixed at compile time. Deflated blobs are inflated
//! once, on their first access, and retained for the rest of the process.

#[cfg(feature = "deflate")]
use std::sync::OnceLock;

pub(crate) struct Blob {
    bytes: &'static [u8],
    #[cfg(feature = "deflate")]
    cell: OnceLock<Vec<u8>>,
}

impl Blob {
    pub(crate) const fn new(bytes: &'static [u8]) -> Self {
        Self {
            bytes,
            #[cfg(feature = "deflate")]
            cell: OnceLock::new(),
        }
    }

    #[cfg(not(feature = "deflate"))]
    pub(crate) fn get(&'static self) -> &'static [u8] {
        self.bytes
    }

    #[cfg(feature = "deflate")]
    pub(crate) fn get(&'static self) -> &'static [u8] {
        self.cell.get_or_init(|| inflate(self.bytes))
    }
}

#[cfg(feature = "deflate")]
fn inflate(blob: &[u8]) -> Vec<u8> {
    miniz_oxide::inflate::decompress_to_vec(blob).expect("failed to inflate bundled data")
}
