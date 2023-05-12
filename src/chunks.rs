use nom::{
    bytes::complete::take,
    sequence::{terminated, tuple},
    IResult,
};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    IDAT(idat::IDATChunk<'a>),
    IEND,
    pHYs(phys::pHYsChunk),
    Unknown(RawChunk<'a>),
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk<'_>> {
    let (input, (raw_length, raw_chunk_type)) = tuple((take(4u32), take(4u32)))(input)?;
    let length = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(raw_length).unwrap());
    if raw_chunk_type == &[73, 72, 68, 82] && length == 13 {
        let (input, ihdr_chunk) = terminated(ihdr::parse_chunk, take(4u32))(input)?;
        Ok((input, Chunk::IHDR(ihdr_chunk)))
    } else if raw_chunk_type == &[112, 72, 89, 115] && length == 9 {
        let (input, phys_chunk) = terminated(phys::parse_chunk, take(4u32))(input)?;
        Ok((input, Chunk::pHYs(phys_chunk)))
    } else if raw_chunk_type == &idat::HEADER {
        let (input, idat_chunk) = terminated(idat::parse_chunk(length), take(4u32))(input)?;
        Ok((input, Chunk::IDAT(idat_chunk)))
    } else if raw_chunk_type == &[73, 69, 78, 68] && length == 0 {
        Ok((&input[4..], Chunk::IEND))
    } else {
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

mod ihdr {
    use nom::{bytes::complete::take, IResult};

    #[derive(Debug)]
    pub struct IHDRChunk {
        width: u32,
        height: u32,
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
        let (input, data) = take(13u32)(input)?;
        let width = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&data[0..4]).unwrap());
        let height = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&data[4..8]).unwrap());
        Ok((
            input,
            IHDRChunk {
                width,
                height,
                bit_depth: data[8],
                color_type: data[9].into(),
                compression_method: data[10],
                filter_method: data[11],
                interlace_method: data[12],
            },
        ))
    }
}

mod phys {
    use nom::{bytes::complete::take, IResult};

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
        let (input, data) = take(9u32)(input)?;
        let x_axis_ppu = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&data[0..4]).unwrap());
        let y_axis_ppu = u32::from_be_bytes(*TryInto::<&[u8; 4]>::try_into(&data[4..8]).unwrap());
        Ok((
            input,
            pHYsChunk {
                x_axis_ppu,
                y_axis_ppu,
                unit_specifier: data[8].into(),
            },
        ))
    }
}

mod idat {
    use nom::{bytes::complete::take, IResult};

    pub const HEADER: [u8; 4] = [73, 68, 65, 84];

    #[derive(Debug)]
    pub struct IDATChunk<'a> {
        data: &'a [u8],
    }

    pub fn parse_chunk(length: u32) -> impl Fn(&[u8]) -> IResult<&[u8], IDATChunk>
where {
        move |i| {
            let (i, data) = take(length)(i)?;
            Ok((i, IDATChunk { data }))
        }
    }
}
