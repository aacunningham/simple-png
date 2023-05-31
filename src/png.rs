use crate::{
    chunks::{
        idat::IDATChunk,
        iend,
        ihdr::{ColorType, IHDRChunk, Interlacing},
        iter_chunks, Chunk, ParseableChunk,
    },
    filters::{filter_scanlines, reconstruct_scanlines},
    pixel::{parse_pixels, Pixel},
    scanlines::{Adam7ScanlineIter, NormalScanline},
};
use anyhow::{anyhow, Context};
use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};
use nom::{bytes::complete::tag, IResult};

fn parse_signature(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"\x89PNG\x0d\x0a\x1a\x0a")(input)
}

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
            interlace_method: Interlacing::None,
        };
        Self {
            header: ihdr,
            pixels,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let header = IHDRChunk {
            height: self.header.height,
            width: self.header.width,
            bit_depth: 16,
            color_type: ColorType::TruecolorWithAlpha,
            filter_method: 0,
            compression_method: 0,
            interlace_method: Interlacing::None,
        };
        let mut data =
            Vec::with_capacity((header.height + header.height * header.width * 4) as usize);
        for line in self.pixels.as_ref().chunks(header.width as usize) {
            data.push(0);
            for p in line {
                data.extend(p.red.to_be_bytes());
                data.extend(p.green.to_be_bytes());
                data.extend(p.blue.to_be_bytes());
                data.extend(p.alpha.to_be_bytes());
            }
        }
        filter_scanlines(&mut data, &header);
        let compressed_data = compress_to_vec_zlib(&data, 9);
        let idat = IDATChunk {
            data: &compressed_data,
        };
        let mut png_data = b"\x89PNG\x0d\x0a\x1a\x0a".to_vec();
        png_data.extend(header.to_bytes());
        png_data.extend(idat.to_bytes());
        png_data.extend(iend::IENDChunk.to_bytes());
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
            match chunk? {
                Chunk::IHDR(ihdr) => header = ihdr,
                Chunk::PLTE(plte) => palette = Some(plte),
                Chunk::IDAT(idat) => {
                    data.extend(idat.data);
                }
                _ => (),
            }
        }
        let mut decompressed_data =
            decompress_to_vec_zlib(&data).context("Failed to decompress image data.")?;
        reconstruct_scanlines(&mut decompressed_data, &header);
        let pixels = match header.interlace_method {
            Interlacing::None => parse_pixels(
                NormalScanline::new(&decompressed_data, &header),
                &header,
                palette.as_ref(),
            )?,
            Interlacing::Adam7 => parse_pixels(
                Adam7ScanlineIter::new(&decompressed_data, &header),
                &header,
                palette.as_ref(),
            )?,
        };
        Ok(PNG { header, pixels })
    }
}
