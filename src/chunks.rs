use nom::{bytes::complete::take, number::complete::be_u32, sequence::tuple, IResult};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    IDAT(idat::IDATChunk<'a>),
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
    let (input, (length, raw_chunk_type)) = tuple((be_u32, take(4u32)))(input)?;
    let (input, (chunk_data, raw_crc)) = tuple((take(length), take(4u32)))(input)?;
    Ok((
        input,
        Chunk::Unknown(RawChunk {
            length,
            chunk_type: raw_chunk_type.try_into().unwrap(),
            chunk_data,
            crc: raw_crc.try_into().unwrap(),
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
    length: u32,
    chunk_type: &'a [u8; 4],
    chunk_data: &'a [u8],
    crc: &'a [u8; 4],
}

pub mod ihdr {
    use nom::{
        bytes::complete::{tag, take},
        sequence::{delimited, tuple},
        IResult,
    };

    pub const HEADER: &[u8; 4] = b"IHDR";

    #[derive(Debug)]
    pub struct IHDRChunk {
        pub width: u32,
        pub height: u32,
        bit_depth: u8,
        color_type: ColorType,
        compression_method: u8,
        filter_method: u8,
        interlace_method: u8,
    }

    #[derive(Debug)]
    enum ColorType {
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

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], IHDRChunk> {
        let (input, chunk_data) = delimited(
            tuple((tag([0, 0, 0, 13]), tag(HEADER))),
            take(13u32),
            take(4u32),
        )(input)?;
        let width = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&chunk_data[0..4]).unwrap());
        let height = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&chunk_data[4..8]).unwrap());
        Ok((
            input,
            IHDRChunk {
                width,
                height,
                bit_depth: chunk_data[8],
                color_type: chunk_data[9].into(),
                compression_method: chunk_data[10],
                filter_method: chunk_data[11],
                interlace_method: chunk_data[12],
            },
        ))
    }
}

mod phys {
    use nom::{
        bytes::complete::{tag, take},
        sequence::{delimited, tuple},
        IResult,
    };

    pub const HEADER: &[u8; 4] = b"pHYs";

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub struct pHYsChunk {
        x_axis_ppu: u32,
        y_axis_ppu: u32,
        unit_specifier: Unit,
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
        let (input, chunk_data) = delimited(
            tuple((tag([0, 0, 0, 9]), tag(HEADER))),
            take(9u32),
            take(4u32),
        )(input)?;
        let x_axis_ppu =
            u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&chunk_data[0..4]).unwrap());
        let y_axis_ppu =
            u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&chunk_data[4..8]).unwrap());
        Ok((
            input,
            pHYsChunk {
                x_axis_ppu,
                y_axis_ppu,
                unit_specifier: chunk_data[8].into(),
            },
        ))
    }
}

pub mod idat {
    use miniz_oxide::inflate::decompress_to_vec_zlib;
    use nom::{
        bytes::complete::{tag, take},
        number::complete::be_u32,
        sequence::terminated,
        IResult,
    };

    pub const HEADER: &[u8; 4] = b"IDAT";

    #[derive(Debug)]
    pub struct IDATChunk<'a> {
        data: &'a [u8],
    }
    impl IDATChunk<'_> {
        pub fn decode_data(&self) -> Vec<u8> {
            decompress_to_vec_zlib(self.data).unwrap()
        }
    }

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], IDATChunk> {
        let (input, length) = terminated(be_u32, tag(HEADER))(input)?;
        let (input, data) = terminated(take(length), take(4u32))(input)?;

        Ok((input, IDATChunk { data }))
    }
}

mod iend {
    use nom::{
        bytes::complete::{tag, take},
        combinator::map,
        sequence::pair,
        IResult,
    };

    pub fn parse_chunk(input: &[u8]) -> IResult<&[u8], ()> {
        map(
            pair(tag([0, 0, 0, 0, b'I', b'E', b'N', b'D']), take(4u32)),
            |_| (),
        )(input)
    }
}
