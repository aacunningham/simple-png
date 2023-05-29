use crate::{crc::calculate_crc, utils::div_ceil};
use nom::{bytes::complete::take, number::complete::be_u32, sequence::tuple, IResult};

use super::ParseableChunk;

#[derive(Debug, Default)]
pub struct IHDRChunk {
    pub width: u32,
    pub height: u32,
    pub(crate) bit_depth: u8,
    pub(crate) color_type: ColorType,
    pub(crate) compression_method: u8,
    pub(crate) filter_method: u8,
    pub(crate) interlace_method: Interlacing,
}
impl IHDRChunk {
    pub(crate) fn filter_width(&self) -> u8 {
        let channel_count = self.color_type.channel_count();
        let sample_width = u8::max(self.bit_depth / 8, 1);
        channel_count * sample_width
    }

    pub(crate) fn pixel_width(&self) -> u8 {
        self.color_type.channel_count() * self.bit_depth
    }

    pub(crate) fn scanline_size(&self) -> usize {
        div_ceil(self.width as usize * self.pixel_width() as usize, 8) + 1
    }
}
impl<'a> ParseableChunk<'a> for IHDRChunk {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"IHDR";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (rest, (width, height, other_bytes)) =
            tuple((be_u32, be_u32, take(5usize)))(chunk_data)?;
        Ok((
            rest,
            IHDRChunk {
                width,
                height,
                bit_depth: other_bytes[0],
                color_type: other_bytes[1].into(),
                compression_method: other_bytes[2],
                filter_method: other_bytes[3],
                interlace_method: other_bytes[4].into(),
            },
        ))
    }

    fn to_bytes(&self) -> Self::Output {
        let mut bytes = vec![0, 0, 0, 13];
        bytes.extend(Self::HEADER);
        bytes.extend(&self.width.to_be_bytes());
        bytes.extend(&self.height.to_be_bytes());
        bytes.extend(&[
            self.bit_depth,
            self.color_type as u8,
            self.compression_method,
            self.filter_method,
            self.interlace_method as u8,
        ]);
        let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
        bytes.extend(crc);
        bytes
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum ColorType {
    #[default]
    Greyscale = 0,
    Truecolor = 2,
    IndexedColor = 3,
    GreyscaleWithAlpha = 4,
    TruecolorWithAlpha = 6,
}
impl From<u8> for ColorType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Greyscale,
            2 => Self::Truecolor,
            3 => Self::IndexedColor,
            4 => Self::GreyscaleWithAlpha,
            6 => Self::TruecolorWithAlpha,
            _ => panic!(),
        }
    }
}
impl ColorType {
    pub(crate) fn channel_count(&self) -> u8 {
        match self {
            Self::Greyscale => 1,
            Self::IndexedColor => 1,
            Self::GreyscaleWithAlpha => 2,
            Self::Truecolor => 3,
            Self::TruecolorWithAlpha => 4,
        }
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum Interlacing {
    #[default]
    None,
    Adam7,
}
impl From<u8> for Interlacing {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Adam7,
            _ => panic!(),
        }
    }
}
