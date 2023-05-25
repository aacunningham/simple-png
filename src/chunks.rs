use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::length_data,
    number::complete::be_u32,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::crc::calculate_crc;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    PLTE(plte::PLTEChunk<'a>),
    IDAT(idat::IDATChunk<&'a [u8]>),
    IEND,
    pHYs(phys::pHYsChunk),
    Unknown(RawChunk<'a>),
}

pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk<'_>> {
    if let Ok((input, chunk)) = ihdr::parse_chunk(input) {
        return Ok((input, Chunk::IHDR(chunk)));
    }
    if let Ok((input, chunk)) = plte::parse_chunk(input) {
        return Ok((input, Chunk::PLTE(chunk)));
    }
    if let Ok((input, chunk)) = phys::parse_chunk(input) {
        return Ok((input, Chunk::pHYs(chunk)));
    }
    if let Ok((input, chunk)) = idat::parse_chunk(input) {
        return Ok((input, Chunk::IDAT(chunk)));
    }
    if let Ok((input, _)) = iend::parse_chunk(input) {
        return Ok((input, Chunk::IEND));
    }
    let (input, (_length, raw_chunk_type)) = tuple((be_u32, take(4usize)))(input)?;
    let (input, (_chunk_data, raw_crc)) = tuple((take(_length), take(4usize)))(input)?;
    Ok((
        input,
        Chunk::Unknown(RawChunk {
            _length,
            _chunk_type: raw_chunk_type.try_into().unwrap(),
            _chunk_data,
            _crc: raw_crc.try_into().unwrap(),
        }),
    ))
}

pub fn iter_chunks(source: &[u8]) -> ChunkIter {
    ChunkIter {
        source,
        finished: false,
    }
}

pub struct ChunkIter<'a> {
    source: &'a [u8],
    finished: bool,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Chunk<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let (rest, chunk) = parse_chunk(self.source).unwrap();
        self.source = rest;
        if matches!(chunk, Chunk::IEND) {
            self.finished = true;
        }
        Some(chunk)
    }
}

#[derive(Debug)]
pub struct RawChunk<'a> {
    _length: u32,
    _chunk_type: &'a [u8; 4],
    _chunk_data: &'a [u8],
    _crc: &'a [u8; 4],
}

pub fn valid_chunk<'a, Error: nom::error::ParseError<&'a [u8]>>(
    header: &'static [u8],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a [u8], Error> {
    move |i: &'a [u8]| {
        let (i, chunk_data) = length_data(map(be_u32, |v| v + 8))(i)?;
        let crc = calculate_crc(chunk_data[0..chunk_data.len() - 4].iter().copied()).to_be_bytes();
        let (_, data) = delimited(tag(header), take(chunk_data.len() - 8), tag(crc))(chunk_data)?;
        Ok((i, data))
    }
}

pub mod ihdr {
    use super::valid_chunk;
    use crate::crc::calculate_crc;
    use nom::{bytes::complete::take, number::complete::be_u32, sequence::tuple, IResult};

    pub const HEADER: &[u8; 4] = b"IHDR";

    #[derive(Debug, Default)]
    pub struct IHDRChunk {
        pub width: u32,
        pub height: u32,
        pub bit_depth: u8,
        pub color_type: ColorType,
        pub compression_method: u8,
        pub filter_method: u8,
        pub interlace_method: u8,
    }
    impl IHDRChunk {
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![0, 0, 0, 13];
            bytes.extend(HEADER);
            bytes.extend(&self.width.to_be_bytes());
            bytes.extend(&self.height.to_be_bytes());
            bytes.extend(&[
                self.bit_depth,
                self.color_type as u8,
                self.compression_method,
                self.filter_method,
                self.interlace_method,
            ]);
            let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
            bytes.extend(crc);
            bytes
        }

        pub fn filter_width(&self) -> u8 {
            let channel_count = self.color_type.channel_count();
            let sample_width = u8::max(self.bit_depth / 8, 1);
            channel_count * sample_width
        }

        pub fn scanline_size(&self) -> usize {
            let pixel_size = self.color_type.channel_count() * self.bit_depth;
            let full_pixel_width = self.width as usize * pixel_size as usize;
            let rem = usize::min(1, full_pixel_width % 8);
            let size = full_pixel_width / 8 + rem as usize + 1;
            size
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub enum ColorType {
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
        pub fn channel_count(&self) -> u8 {
            match self {
                Self::Greyscale => 1,
                Self::IndexedColor => 1,
                Self::GreyscaleWithAlpha => 2,
                Self::Truecolor => 3,
                Self::TruecolorWithAlpha => 4,
            }
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], IHDRChunk> {
        let (rest, chunk_data) = valid_chunk(HEADER)(input)?;
        let (_, (width, height, other_bytes)) = tuple((be_u32, be_u32, take(5usize)))(chunk_data)?;
        Ok((
            rest,
            IHDRChunk {
                width,
                height,
                bit_depth: other_bytes[0],
                color_type: other_bytes[1].into(),
                compression_method: other_bytes[2],
                filter_method: other_bytes[3],
                interlace_method: other_bytes[4],
            },
        ))
    }
}

pub mod plte {
    use super::valid_chunk;
    use nom::IResult;

    pub const HEADER: &[u8; 4] = b"PLTE";

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub struct PLTEChunk<'a> {
        colors: &'a [u8],
    }
    impl PLTEChunk<'_> {
        pub fn get_color(&self, index: u8) -> Option<(u8, u8, u8)> {
            let index = index as usize;
            Some((
                *self.colors.get(index)?,
                *self.colors.get(index + 1)?,
                *self.colors.get(index + 2)?,
            ))
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], PLTEChunk> {
        let (rest, colors) = valid_chunk(HEADER)(input)?;
        Ok((rest, PLTEChunk { colors }))
    }
}

mod phys {
    use super::valid_chunk;
    use nom::{
        number::complete::{be_u32, u8},
        sequence::tuple,
        IResult,
    };

    pub const HEADER: &[u8; 4] = b"pHYs";

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub struct pHYsChunk {
        _x_axis_ppu: u32,
        _y_axis_ppu: u32,
        _unit_specifier: u8,
    }

    #[derive(Debug)]
    enum Unit {
        Unknown,
        Meter,
    }
    impl From<u8> for Unit {
        fn from(value: u8) -> Self {
            if value == 1 {
                Self::Meter
            } else {
                Self::Unknown
            }
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], pHYsChunk> {
        let (rest, chunk_data) = valid_chunk(HEADER)(input)?;
        let (_, (_x_axis_ppu, _y_axis_ppu, _unit_specifier)) =
            tuple((be_u32, be_u32, u8))(chunk_data)?;
        Ok((
            rest,
            pHYsChunk {
                _x_axis_ppu,
                _y_axis_ppu,
                _unit_specifier,
            },
        ))
    }
}

pub mod idat {
    use crate::crc::calculate_crc;
    use nom::IResult;

    use super::valid_chunk;

    pub const HEADER: &[u8; 4] = b"IDAT";

    #[derive(Debug)]
    pub struct IDATChunk<T> {
        pub data: T,
    }
    impl<T> IDATChunk<T>
    where
        T: AsRef<[u8]>,
    {
        pub fn to_bytes(&self) -> Vec<u8> {
            let len = self.data.as_ref().len() as u32;
            let mut bytes = len.to_be_bytes().to_vec();
            bytes.extend(HEADER);
            bytes.extend(self.data.as_ref());
            let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
            bytes.extend(crc);
            bytes
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], IDATChunk<&[u8]>> {
        let (rest, data) = valid_chunk(HEADER)(input)?;
        Ok((rest, IDATChunk { data }))
    }
}

pub mod iend {
    use nom::{combinator::map, IResult};

    use crate::crc::calculate_crc;

    use super::valid_chunk;

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], ()> {
        map(valid_chunk(b"IEND"), |_| ())(input)
    }

    pub fn write_end() -> [u8; 12] {
        let mut data = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0];
        let crc = calculate_crc(data[4..8].iter().copied()).to_be_bytes();
        for (i, b) in crc.into_iter().enumerate() {
            data[i + 8] = b;
        }
        data
    }
}
