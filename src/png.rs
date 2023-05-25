use anyhow::anyhow;

use crate::{
    chunks::{
        idat::IDATChunk,
        iend,
        ihdr::{ColorType, IHDRChunk},
        iter_chunks,
        plte::PLTEChunk,
        Chunk,
    },
    decoder::parse_signature,
    image_data::{compress_data, decompress_data},
    pixel::{parse_pixels, Pixel},
};

#[derive(Debug)]
pub struct PNG<T>
where
    T: AsRef<[Pixel]>,
{
    pub header: IHDRChunk,
    pub pixels: T,
}

impl<T> PNG<T>
where
    T: AsRef<[Pixel]>,
{
    pub fn new(height: u32, width: u32, pixels: T) -> Self {
        let ihdr = IHDRChunk {
            height,
            width,
            bit_depth: 8,
            color_type: ColorType::TruecolorWithAlpha,
            filter_method: 0,
            compression_method: 0,
            interlace_method: 0,
        };
        Self {
            header: ihdr,
            pixels,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let ihdr = IHDRChunk {
            height: self.header.height,
            width: self.header.width,
            bit_depth: 16,
            color_type: ColorType::TruecolorWithAlpha,
            filter_method: 0,
            compression_method: 0,
            interlace_method: 0,
        };
        let mut data = Vec::with_capacity((ihdr.height + ihdr.height * ihdr.width * 4) as usize);
        for line in self.pixels.as_ref().chunks(ihdr.width as usize) {
            data.push(0);
            for p in line {
                data.extend(p.red.to_be_bytes());
                data.extend(p.green.to_be_bytes());
                data.extend(p.blue.to_be_bytes());
                data.extend(p.alpha.to_be_bytes());
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
}

impl PNG<Vec<Pixel>> {
    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let (rest, _) = parse_signature(bytes)
            .or(Err(anyhow!("Data doesn't start with expected signature")))?;
        let mut header = IHDRChunk::default();
        let mut palette = None;
        let mut data = vec![];
        for chunk in iter_chunks(rest) {
            match chunk {
                Chunk::IHDR(ihdr) => header = ihdr,
                Chunk::PLTE(plte) => palette = Some(plte),
                Chunk::IDAT(idat) => {
                    data.extend(idat.data);
                }
                _ => (),
            }
        }
        let scanlines = decompress_data(&data, &header);
        let scanline_size = header.scanline_size();
        let pixels = parse_pixels(
            scanlines.chunks(scanline_size).map(|sl| &sl[1..]),
            &header,
            palette.as_ref(),
        );
        Ok(PNG { header, pixels })
    }
}
