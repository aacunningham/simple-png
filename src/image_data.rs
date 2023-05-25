use anyhow::Context;
use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};

use crate::{chunks::ihdr::IHDRChunk, filters::Filter};

pub fn compress_data(data: &mut [u8], ihdr: &IHDRChunk) -> Vec<u8> {
    let pixel_width = ihdr.filter_width() as usize;
    let scanline_size = ihdr.scanline_size();

    // Handle first scanline as special case
    data[0] = Filter::Sub as u8;
    for i in data[1..pixel_width + 1].iter_mut() {
        *i = Filter::Sub.filter(*i, 0, 0, 0);
    }
    for i in (pixel_width + 1)..scanline_size {
        data[i] = Filter::Sub.filter(data[i], data[i - pixel_width], 0, 0)
    }

    // Remaining scanlines
    for i in 1..ihdr.height as usize {
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

pub fn decompress_data(compressed_data: &[u8], ihdr: &IHDRChunk) -> anyhow::Result<Vec<u8>> {
    let mut data =
        decompress_to_vec_zlib(compressed_data).context("Failed to decompress image data.")?;
    let pixel_width = ihdr.filter_width() as usize;
    let scanline_size = ihdr.scanline_size();

    // Handle first scanline as special case
    let filter = Filter::try_from(data[0])?;
    for b in data[1..(pixel_width + 1)].iter_mut() {
        *b = filter.reconstruct(*b, 0, 0, 0);
    }
    for i in (pixel_width + 1)..scanline_size {
        data[i] = filter.reconstruct(data[i], data[i - pixel_width], 0, 0);
    }

    // Remaining scanlines
    for i in 1..ihdr.height as usize {
        let filter = Filter::try_from(data[i * scanline_size])?;
        let (start, stop) = (i * scanline_size + 1, (i + 1) * scanline_size);
        for j in start..(start + pixel_width) {
            data[j] = filter.reconstruct(data[j], 0, data[j - scanline_size], 0);
        }
        for j in (start + pixel_width)..stop {
            let a = data[j - pixel_width];
            let b = data[j - scanline_size];
            let c = data[j - pixel_width - scanline_size];
            data[j] = filter.reconstruct(data[j], a, b, c);
        }
    }
    Ok(data)
}
