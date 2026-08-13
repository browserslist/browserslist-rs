//! Bundled byte arrays, stored either verbatim or run-length encoded, and either way
//! optionally deflated.
//!
//! Without the `deflate` feature a verbatim blob is the bytes themselves and costs
//! nothing at runtime; leaving the data uncompressed suits binaries that get packed as
//! a whole afterwards. With the feature the bytes are a raw deflate stream that is
//! inflated on first access and kept for the rest of the process.
//!
//! Deflate rather than a stronger codec because the decoder has to be paid for too.
//! Measured against this data, zstd, brotli and xz each compress better but their
//! decoders cost 5 to 8 times more code than miniz_oxide's inflate, and every one of
//! them ends up larger overall.
//!
//! Run-length encoding is chosen per array by the generator, which takes it only where
//! it shrinks the array by at least a third. Several of these columns are sorted, or
//! are the high plane of an index that never reaches its second byte, and collapse to
//! almost nothing; it costs a heap copy at load time, so arrays without real runs keep
//! their verbatim form.

use std::sync::OnceLock;

pub(crate) struct Blob {
    bytes: &'static [u8],
    /// One count per byte of `bytes`, or empty when the array is stored verbatim.
    counts: &'static [u8],
    cell: OnceLock<Vec<u8>>,
}

impl Blob {
    pub(crate) const fn new(bytes: &'static [u8]) -> Self {
        Self {
            bytes,
            counts: &[],
            cell: OnceLock::new(),
        }
    }

    pub(crate) const fn rle(values: &'static [u8], counts: &'static [u8]) -> Self {
        Self {
            bytes: values,
            counts,
            cell: OnceLock::new(),
        }
    }

    #[cfg(not(feature = "deflate"))]
    pub(crate) fn get(&'static self) -> &'static [u8] {
        if self.counts.is_empty() {
            self.bytes
        } else {
            self.cell.get_or_init(|| unrle(self.bytes, self.counts))
        }
    }

    #[cfg(feature = "deflate")]
    pub(crate) fn get(&'static self) -> &'static [u8] {
        self.cell.get_or_init(|| {
            let bytes = inflate(self.bytes);
            if self.counts.is_empty() {
                bytes
            } else {
                unrle(&bytes, &inflate(self.counts))
            }
        })
    }
}

/// Undoes the run-length encoding `write_blob` in generate-data applies: one run per
/// value and count pair, with runs too long for a count byte split across several pairs.
fn unrle(values: &[u8], counts: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(counts.iter().map(|count| usize::from(*count)).sum());
    for (value, count) in values.iter().zip(counts) {
        bytes.extend(std::iter::repeat_n(*value, usize::from(*count)));
    }
    bytes
}

/// Inflates a blob written by `write_blob` in generate-data: the inflated length as a
/// little-endian `u32`, then a raw deflate stream. Knowing the length up front means a
/// single pass into an exactly sized buffer, which keeps miniz_oxide's buffer-growing
/// wrapper -- and the code size that comes with it -- out of the binary.
#[cfg(feature = "deflate")]
fn inflate(blob: &[u8]) -> Vec<u8> {
    use miniz_oxide::inflate::{
        TINFLStatus,
        core::{
            DecompressorOxide, decompress, inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
        },
    };

    let (len, compressed) = blob.split_at(4);
    let len = u32::from_le_bytes([len[0], len[1], len[2], len[3]]) as usize;

    let mut out = vec![0; len];
    let (status, _, written) = decompress(
        &mut DecompressorOxide::default(),
        compressed,
        &mut out,
        0,
        TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
    );
    assert!(
        status == TINFLStatus::Done && written == len,
        "failed to inflate bundled data"
    );
    out
}
