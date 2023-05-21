use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};

use crate::{chunks::ihdr::IHDRChunk, filters::Filter};

pub fn compress_data(data: &mut [u8], ihdr: &IHDRChunk) -> Vec<u8> {
    let channel_count = ihdr.color_type.channel_count() as usize;
    let scanline_size = channel_count * ihdr.width as usize + 1;

    // Handle first scanline as special case
    data[0] = Filter::Sub as u8;
    for i in 1..(channel_count + 1) {
        data[i] = Filter::Sub.filter(data[i], 0, 0, 0);
    }
    for i in (channel_count + 1)..scanline_size {
        data[i] = Filter::Sub.filter(data[i], data[i - channel_count], 0, 0)
    }

    // Remaining scanlines
    for i in 1..ihdr.height as usize {
        data[i * scanline_size] = Filter::Sub as u8;
        let (start, stop) = (i * scanline_size + 1, (i + 1) * scanline_size);
        for j in start..(start + channel_count) {
            data[j] = Filter::Sub.filter(data[j], 0, data[j - scanline_size], 0);
        }
        for j in (start + channel_count)..stop {
            let a = data[j - channel_count];
            let b = data[j - scanline_size];
            let c = data[j - channel_count - scanline_size];
            data[j] = Filter::Sub.filter(data[j], a, b, c);
        }
    }
    compress_to_vec_zlib(&data, 8)
}

pub fn decompress_data(compressed_data: &[u8], ihdr: &IHDRChunk) -> Vec<u8> {
    let mut data = decompress_to_vec_zlib(compressed_data).unwrap();
    let channel_count = ihdr.color_type.channel_count() as usize;
    let scanline_size = channel_count * ihdr.width as usize + 1;

    // Handle first scanline as special case
    let filter = Filter::try_from(data[0]).unwrap();
    for b in data[1..(channel_count + 1)].iter_mut() {
        *b = filter.reconstruct(*b, 0, 0, 0);
    }
    for i in (channel_count + 1)..scanline_size {
        data[i] = filter.reconstruct(data[i], data[i - channel_count], 0, 0);
    }

    // Remaining scanlines
    for i in 1..ihdr.height as usize {
        let filter = Filter::try_from(data[i * scanline_size]).unwrap();
        let (start, stop) = (i * scanline_size + 1, (i + 1) * scanline_size);
        for j in start..(start + channel_count) {
            data[j] = filter.reconstruct(data[j], 0, data[j - scanline_size], 0);
        }
        for j in (start + channel_count)..stop {
            let a = data[j - channel_count];
            let b = data[j - scanline_size];
            let c = data[j - channel_count - scanline_size];
            data[j] = filter.reconstruct(data[j], a, b, c);
        }
    }
    data
}
