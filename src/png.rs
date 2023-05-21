use anyhow::{anyhow, bail};

use crate::{
    chunks::{
        idat::IDATChunk,
        iend,
        ihdr::{ColorType, IHDRChunk},
        iter_chunks, Chunk,
    },
    decoder::parse_signature,
    image_data::{compress_data, decompress_data},
    pixel::Pixel,
};

#[derive(Debug)]
pub struct PNG {
    pub header: IHDRChunk,
    pub pixels: Vec<Pixel>,
}

impl PNG {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let (rest, _) = parse_signature(bytes)
            .or(Err(anyhow!("Data doesn't start with expected signature")))?;
        let mut header = IHDRChunk::default();
        let mut data = vec![];
        for chunk in iter_chunks(rest) {
            match chunk {
                Chunk::IHDR(ihdr) => {
                    header = ihdr;
                }
                Chunk::IDAT(idat) => {
                    data.extend(idat.data);
                }
                _ => (),
            }
        }
        if header.bit_depth != 8 {
            bail!("Only bit depth of 8 is supported");
        }
        let scanlines = decompress_data(&data, &header);

        let channels = header.color_type.channel_count() as usize;
        let scanline_size = header.width as usize * channels + 1;
        let mut pixels = Vec::with_capacity(header.width as usize * header.height as usize);
        for scanline in scanlines.chunks(scanline_size) {
            for pixel in scanline[1..].chunks(channels) {
                pixels.push(Pixel {
                    red: pixel[0],
                    green: pixel[1],
                    blue: pixel[2],
                    alpha: pixel[3],
                })
            }
        }

        Ok(PNG { header, pixels })
    }
}

pub fn encode(height: u32, width: u32, pixel: &[Pixel]) -> Vec<u8> {
    let ihdr = IHDRChunk {
        height,
        width,
        bit_depth: 8,
        color_type: ColorType::TrueColorWithAlpha,
        filter_method: 0,
        compression_method: 0,
        interlace_method: 0,
    };
    let mut data = Vec::with_capacity((height + height * width * 4) as usize);
    for line in pixel.chunks(width as usize) {
        data.push(0);
        for p in line {
            data.push(p.red);
            data.push(p.green);
            data.push(p.blue);
            data.push(p.alpha);
        }
    }
    let compressed_data = compress_data(&mut data, &ihdr);
    let idat = IDATChunk {
        data: &compressed_data,
    };
    let mut png_data = b"\x89PNG\x0d\x0a\x1a\x0a".to_vec();
    png_data.extend(ihdr.to_bytes());
    png_data.extend(idat.to_bytes());
    png_data.extend(iend::write_end());
    png_data
}
