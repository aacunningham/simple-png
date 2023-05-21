use nom::{bytes::complete::take, number::complete::be_u32, sequence::tuple, IResult};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    IDAT(idat::IDATChunk<&'a [u8]>),
    IEND,
    pHYs(phys::pHYsChunk),
    Unknown(RawChunk<'a>),
}

pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk<'_>> {
    if let Ok((input, chunk)) = ihdr::parse_chunk(input) {
        return Ok((input, Chunk::IHDR(chunk)));
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
    let (input, (_length, raw_chunk_type)) = tuple((be_u32, take(4u32)))(input)?;
    let (input, (_chunk_data, raw_crc)) = tuple((take(_length), take(4u32)))(input)?;
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

pub mod ihdr {
    use nom::{
        bytes::complete::{tag, take},
        number::complete::be_u32,
        sequence::{delimited, tuple},
        IResult,
    };

    use crate::crc::calculate_crc;

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

    #[derive(Debug, Default, Clone, Copy)]
    pub enum ColorType {
        #[default]
        Greyscale = 0,
        Truecolor = 2,
        IndexedColor = 3,
        GreyscaleWithAlpha = 4,
        TrueColorWithAlpha = 6,
    }
    impl From<u8> for ColorType {
        fn from(value: u8) -> Self {
            match value {
                0 => Self::Greyscale,
                2 => Self::Truecolor,
                3 => Self::IndexedColor,
                4 => Self::GreyscaleWithAlpha,
                6 => Self::TrueColorWithAlpha,
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
                Self::TrueColorWithAlpha => 4,
            }
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], IHDRChunk> {
        let (input, (width, height, chunk_data)) = delimited(
            tuple((tag([0, 0, 0, 13]), tag(HEADER))),
            tuple((be_u32, be_u32, take(5u32))),
            take(4u32),
        )(input)?;
        Ok((
            input,
            IHDRChunk {
                width,
                height,
                bit_depth: chunk_data[0],
                color_type: chunk_data[1].into(),
                compression_method: chunk_data[2],
                filter_method: chunk_data[3],
                interlace_method: chunk_data[4],
            },
        ))
    }

    impl IHDRChunk {
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![0, 0, 0, 13];
            bytes.extend(HEADER);
            bytes.extend(&self.width.to_be_bytes());
            bytes.extend(&self.height.to_be_bytes());
            bytes.push(self.bit_depth);
            bytes.push(self.color_type as u8);
            bytes.push(self.compression_method);
            bytes.push(self.filter_method);
            bytes.push(self.interlace_method);
            let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
            bytes.extend(crc);
            bytes
        }
    }
}

pub mod plte {
    use nom::{
        bytes::complete::{tag, take},
        number::complete::be_u32,
        sequence::terminated,
        IResult,
    };

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
        let (input, length) = terminated(be_u32, tag(HEADER))(input)?;
        let (input, colors) = take(length)(input)?;
        Ok((input, PLTEChunk { colors }))
    }
}

mod phys {
    use nom::{
        bytes::complete::{tag, take},
        number::complete::{be_u32, u8},
        sequence::{delimited, tuple},
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
        let (input, (_x_axis_ppu, _y_axis_ppu, _unit_specifier)) = delimited(
            tuple((tag([0, 0, 0, 9]), tag(HEADER))),
            tuple((be_u32, be_u32, u8)),
            take(4u32),
        )(input)?;
        Ok((
            input,
            pHYsChunk {
                _x_axis_ppu,
                _y_axis_ppu,
                _unit_specifier: _unit_specifier.into(),
            },
        ))
    }
}

pub mod idat {
    use miniz_oxide::deflate::compress_to_vec_zlib;
    use miniz_oxide::inflate::decompress_to_vec_zlib;
    use nom::{
        bytes::complete::{tag, take},
        number::complete::be_u32,
        sequence::terminated,
        IResult,
    };

    use crate::crc::calculate_crc;

    pub const HEADER: &[u8; 4] = b"IDAT";

    #[derive(Debug)]
    pub struct IDATChunk<T> {
        pub data: T,
    }
    impl<'a, T> IDATChunk<T>
    where
        T: AsRef<[u8]>,
    {
        pub fn to_bytes(self) -> Vec<u8> {
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
        let (input, length) = terminated(be_u32, tag(HEADER))(input)?;
        let (input, data) = terminated(take(length), take(4u32))(input)?;

        Ok((input, IDATChunk { data }))
    }
}

pub mod iend {
    use nom::{
        bytes::complete::{tag, take},
        combinator::map,
        sequence::pair,
        IResult,
    };

    use crate::crc::calculate_crc;

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], ()> {
        map(
            pair(tag([0, 0, 0, 0, b'I', b'E', b'N', b'D']), take(4u32)),
            |_| (),
        )(input)
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
