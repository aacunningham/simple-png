use anyhow::Context;
use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};

use crate::{
    chunks::ihdr::IHDRChunk,
    filters::{reconstruct_scanlines, Filter},
};

pub(crate) fn compress_data(data: &mut [u8], header: &IHDRChunk) -> Vec<u8> {
    let pixel_width = header.filter_width() as usize;
    let scanline_size = header.scanline_size();

    // Handle first scanline as special case
    data[0] = Filter::Sub as u8;
    for i in data[1..pixel_width + 1].iter_mut() {
        *i = Filter::Sub.filter(*i, 0, 0, 0);
    }
    for i in (pixel_width + 1)..scanline_size {
        data[i] = Filter::Sub.filter(data[i], data[i - pixel_width], 0, 0)
    }

    // Remaining scanlines
    for i in 1..header.height as usize {
        data[i * scanline_size] = Filter::Sub as u8;
        let (start, stop) = (i * scanline_size + 1, (i + 1) * scanline_size);
        for j in start..(start + pixel_width) {
            data[j] = Filter::Sub.filter(data[j], 0, data[j - pixel_width], 0);
        }
        for j in (start + pixel_width)..stop {
            let a = data[j - pixel_width];
            let b = data[j - scanline_size];
            let c = data[j - pixel_width - scanline_size];
            data[j] = Filter::Sub.filter(data[j], a, b, c);
        }
    }
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
