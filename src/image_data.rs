use anyhow::Context;
use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};

use crate::{
    chunks::ihdr::IHDRChunk,
    filters::{filter_scanlines, reconstruct_scanlines},
};

pub(crate) fn compress_data(data: &mut [u8], header: &IHDRChunk) -> Vec<u8> {
    filter_scanlines(data, header);
    compress_to_vec_zlib(data, 9)
}

pub(crate) fn decompress_data(
    compressed_data: &[u8],
    header: &IHDRChunk,
) -> anyhow::Result<Vec<u8>> {
    let mut data =
        decompress_to_vec_zlib(compressed_data).context("Failed to decompress image data.")?;
    reconstruct_scanlines(&mut data, header);

    Ok(data)
}
