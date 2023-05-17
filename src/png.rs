use anyhow::{anyhow, bail};

use crate::{
    chunks::{
        idat::IDATChunk,
        iend,
        ihdr::{ColorType, IHDRChunk},
        iter_chunks, Chunk,
    },
    decoder::parse_signature,
    filters::Filter,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}
impl Pixel {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}
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
                    data.extend(idat.decode_data());
                }
                _ => (),
            }
        }
        dbg!(&data);
        if header.bit_depth != 8 {
            bail!("Only bit depth of 8 is supported");
        }
        let channels = header.color_type.channel_count() as usize;
        let scanline_size = header.width as usize * channels + 1;
        let mut pixels = Vec::with_capacity(header.width as usize * header.height as usize);

        let (filter, rest) = (
            Filter::try_from(data[0]).unwrap(),
            &mut data[1..scanline_size],
        );
        for b in rest[0..channels].iter_mut() {
            *b = filter.reconstruct(*b, 0, 0, 0);
        }
        for i in channels..rest.len() {
            rest[i] = filter.reconstruct(rest[i], rest[i - channels], 0, 0);
        }
        for p in rest.chunks(channels) {
            pixels.push(Pixel {
                red: p[0],
                green: p[1],
                blue: p[2],
                alpha: p[3],
            });
        }
        for i in 1..header.height as usize {
            let filter = Filter::try_from(data[i * scanline_size]).unwrap();
            let (start, stop) = (i * scanline_size + 1, (i + 1) * scanline_size);
            for j in start..(start + channels) {
                data[j] = filter.reconstruct(data[j], 0, data[j - scanline_size], 0);
            }
            for j in (start + channels)..stop {
                let a = data[j - channels];
                let b = data[j - scanline_size];
                let c = data[j - channels - scanline_size];
                data[j] = filter.reconstruct(data[j], a, b, c);
            }
            for p in data[start..stop].chunks(channels) {
                pixels.push(Pixel {
                    red: p[0],
                    green: p[1],
                    blue: p[2],
                    alpha: p[3],
                });
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
    filter_idat(&mut data, &ihdr);
    let idat = IDATChunk::encode_data(data);
    let mut png_data = b"\x89PNG\x0d\x0a\x1a\x0a".to_vec();
    png_data.extend(ihdr.to_bytes());
    png_data.extend(idat.to_bytes());
    png_data.extend(iend::write_end());
    png_data
}

fn filter_idat(data: &mut [u8], ihdr: &IHDRChunk) {
    let channel_count = ihdr.color_type.channel_count() as usize;
    let scanline_size = channel_count * ihdr.width as usize + 1;

    // Handle first row as special case
    data[0] = Filter::Sub as u8;
    for i in 1..(channel_count + 1) {
        data[i] = Filter::Sub.filter(data[i], 0, 0, 0);
    }
    for i in (channel_count + 1)..scanline_size {
        data[i] = Filter::Sub.filter(data[i], data[i - channel_count], 0, 0)
    }

    // Remaining rows
    for i in 1..ihdr.height as usize {
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
}
