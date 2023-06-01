use anyhow::anyhow;

use crate::{
    chunks::ihdr::{IHDRChunk, Interlacing},
    interlacing::Adam7Iter,
    utils::div_ceil,
};

pub(crate) enum Filter {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}
impl Filter {
    #[allow(unused)]
    pub(crate) fn filter(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x.wrapping_sub(a),
            Filter::Up => x.wrapping_sub(b),
            Filter::Average => {
                let a = a as u16;
                let b = b as u16;
                x.wrapping_sub(((a + b) / 2) as u8)
            }
            Filter::Paeth => x.wrapping_sub(paeth_predictor(a, b, c)),
        }
    }

    pub(crate) fn reconstruct(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x.wrapping_add(a),
            Filter::Up => x.wrapping_add(b),
            Filter::Average => {
                let a = a as u16;
                let b = b as u16;
                x.wrapping_add(((a + b) / 2) as u8)
            }
            Filter::Paeth => x.wrapping_add(paeth_predictor(a, b, c)),
        }
    }
}
impl TryFrom<u8> for Filter {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Up),
            3 => Ok(Self::Average),
            4 => Ok(Self::Paeth),
            i => Err(anyhow!("Filter type {i} is unknown.")),
        }
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = i16::abs(p - a as i16);
    let pb = i16::abs(p - b as i16);
    let pc = i16::abs(p - c as i16);
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

pub(crate) fn reconstruct_scanlines(image_data: &mut [u8], header: &IHDRChunk) {
    let pixel_width = header.color_type.channel_count() * header.bit_depth;
    match header.interlace_method {
        Interlacing::None => {
            let scanline_length = div_ceil(header.width as usize * pixel_width as usize, 8) + 1;
            inner_reconstruct_scanlines(
                image_data,
                scanline_length,
                header.height as usize,
                header.filter_width() as usize,
            );
        }
        Interlacing::Adam7 => {
            let mut image_data_index = 0;
            for sub_image in Adam7Iter::new(header.width as usize, header.height as usize) {
                let scanline_length = div_ceil(sub_image.width * pixel_width as usize, 8) + 1;
                image_data_index += inner_reconstruct_scanlines(
                    &mut image_data
                        [image_data_index..(image_data_index + scanline_length * sub_image.height)],
                    scanline_length,
                    sub_image.height,
                    header.filter_width() as usize,
                );
            }
        }
    };
}

fn inner_reconstruct_scanlines(
    image_data: &mut [u8],
    scanline_length: usize,
    line_count: usize,
    filter_width: usize,
) -> usize {
    assert!(image_data.len() % scanline_length == 0);
    log::info!("{:?}", filter_width);

    // Handle first scanline as special case
    log::info!("Scanline: {:?}", &image_data[0..scanline_length]);
    let filter = Filter::try_from(image_data[0]).unwrap();
    for b in image_data[1..(filter_width + 1)].iter_mut() {
        *b = filter.reconstruct(*b, 0, 0, 0);
    }
    for i in (filter_width + 1)..scanline_length {
        image_data[i] = filter.reconstruct(image_data[i], image_data[i - filter_width], 0, 0);
    }
    log::info!("Scanline: {:?}", &image_data[0..scanline_length]);

    // Remaining scanlines
    for i in 1..line_count {
        let filter = Filter::try_from(image_data[i * scanline_length]).unwrap();
        let (start, stop) = (i * scanline_length + 1, (i + 1) * scanline_length);
        for j in start..(start + filter_width) {
            image_data[j] =
                filter.reconstruct(image_data[j], 0, image_data[j - scanline_length], 0);
        }
        for j in (start + filter_width)..stop {
            let a = image_data[j - filter_width];
            let b = image_data[j - scanline_length];
            let c = image_data[j - filter_width - scanline_length];
            image_data[j] = filter.reconstruct(image_data[j], a, b, c);
        }
        // log::info!("Scanline: {:?}", &image_data[(i * scanline_length)..stop]);
    }
    scanline_length * line_count
}

pub(crate) fn filter_scanlines(image_data: &mut [u8], header: &IHDRChunk) {
    let pixel_width = header.color_type.channel_count() * header.bit_depth;
    match header.interlace_method {
        Interlacing::None => {
            let scanline_length = div_ceil(header.width as usize * pixel_width as usize, 8) + 1;
            inner_filter_scanlines(
                image_data,
                scanline_length,
                header.height as usize,
                header.filter_width() as usize,
            );
        }
        Interlacing::Adam7 => {
            let mut image_data_index = 0;
            for sub_image in Adam7Iter::new(header.width as usize, header.height as usize) {
                let scanline_length = div_ceil(sub_image.width * pixel_width as usize, 8) + 1;
                image_data_index += inner_filter_scanlines(
                    &mut image_data
                        [image_data_index..(image_data_index + scanline_length * sub_image.height)],
                    scanline_length,
                    sub_image.height,
                    header.filter_width() as usize,
                );
            }
        }
    };
}

fn inner_filter_scanlines(
    image_data: &mut [u8],
    scanline_length: usize,
    line_count: usize,
    filter_width: usize,
) -> usize {
    assert!(image_data.len() % scanline_length == 0);

    // Start from the bottom
    for i in (1..line_count).rev() {
        image_data[i * scanline_length] = Filter::Sub as u8;
        let (start, stop) = (i * scanline_length + 1, (i + 1) * scanline_length);
        // log::info!("Scanline: {:?}", &image_data[(i * scanline_length)..stop]);
        for j in start..(start + filter_width) {
            image_data[j] = Filter::Sub.filter(image_data[j], 0, image_data[j - filter_width], 0);
        }
        for j in ((start + filter_width)..stop).rev() {
            let a = image_data[j - filter_width];
            let b = image_data[j - scanline_length];
            let c = image_data[j - filter_width - scanline_length];
            image_data[j] = Filter::Sub.filter(image_data[j], a, b, c);
        }
    }

    // log::info!("Scanline: {:?}", &image_data[0..scanline_length]);
    // "First" scanline can be treated differently
    for i in ((filter_width + 1)..scanline_length).rev() {
        image_data[i] = Filter::Sub.filter(image_data[i], image_data[i - filter_width], 0, 0)
    }
    // Special case for the first pixel/byte.
    image_data[0] = Filter::Sub as u8;
    for i in image_data[1..filter_width + 1].iter_mut() {
        *i = Filter::Sub.filter(*i, 0, 0, 0);
    }
    scanline_length * line_count
}

#[cfg(test)]
mod tests {
    use super::{inner_filter_scanlines, inner_reconstruct_scanlines, Filter};

    #[test]
    fn reconstruct_undoes_filter() {
        let data = &mut [
            1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1,
            0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255,
        ];
        let orig_copy = data.clone();
        inner_filter_scanlines(data, 17, 2, 8);
        assert_eq!(
            data,
            &[
                1, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
                0, // End scanline
                1, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
        inner_reconstruct_scanlines(data, 17, 2, 8);
        assert_eq!(data, &orig_copy);
    }

    #[test]
    fn real_test() {
        let data = &mut [
            1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 1, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255,
        ];
        let orig_copy = data.clone();
        inner_filter_scanlines(data, 257, 2, 8);
        inner_reconstruct_scanlines(data, 257, 2, 8);
        assert_eq!(data, &orig_copy);
    }

    #[test]
    fn average_actually_averages_a_and_b() {
        assert_eq!(Filter::Average.filter(50, 50, 50, 0), 0);
        assert_eq!(Filter::Average.filter(75, 100, 50, 0), 0);
        assert_eq!(Filter::Average.filter(254, 255, 253, 0), 0);
    }
}
