use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::length_data,
    number::complete::be_u32,
    sequence::{terminated, tuple},
    IResult,
};

use crate::crc::calculate_crc;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    PLTE(plte::PLTEChunk<'a>),
    IDAT(idat::IDATChunk<&'a [u8]>),
    IEND,
    pHYs(phys::pHYsChunk),
    Unknown(RawChunk<'a>),
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk<'_>> {
    let (rest, (header, chunk_data)) = valid_chunk(input)?;
    match header {
        ihdr::HEADER => Ok((rest, Chunk::IHDR(ihdr::parse_data(chunk_data)?.1))),
        plte::HEADER => Ok((rest, Chunk::PLTE(plte::parse_data(chunk_data)?.1))),
        phys::HEADER => Ok((rest, Chunk::pHYs(phys::parse_data(chunk_data)?.1))),
        idat::HEADER => Ok((rest, Chunk::IDAT(idat::parse_data(chunk_data)?.1))),
        iend::HEADER => Ok((rest, Chunk::IEND)),
        _ => Ok((
            rest,
            Chunk::Unknown(RawChunk {
                _chunk_type: header,
                _chunk_data: chunk_data,
            }),
        )),
    }
}

pub(crate) fn iter_chunks(source: &[u8]) -> ChunkIter {
    ChunkIter {
        source,
        finished: false,
    }
}

pub(crate) struct ChunkIter<'a> {
    source: &'a [u8],
    finished: bool,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = anyhow::Result<Chunk<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match parse_chunk(self.source) {
            Ok((rest, chunk)) => {
                self.source = rest;
                if matches!(chunk, Chunk::IEND) {
                    self.finished = true;
                }
                Some(Ok(chunk))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e.to_owned().into()))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct RawChunk<'a> {
    _chunk_type: &'a [u8; 4],
    _chunk_data: &'a [u8],
}

fn valid_chunk<'a, Error: nom::error::ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8; 4], &'a [u8]), Error> {
    let (header_length, crc_length) = (4, 4);
    let (input, chunk_data) = length_data(map(be_u32, |v| v + header_length + crc_length))(input)?;
    let crc = calculate_crc(
        chunk_data[0..chunk_data.len() - crc_length as usize]
            .iter()
            .copied(),
    )
    .to_be_bytes();
    let (_, data) = tuple((
        map(take(header_length), |v: &[u8]| {
            v.try_into().expect("4 bytes should have been taken")
        }),
        terminated(
            take(chunk_data.len() - (header_length + crc_length) as usize),
            tag(crc),
        ),
    ))(chunk_data)?;
    Ok((input, data))
}

pub(crate) mod ihdr {
    use crate::{crc::calculate_crc, utils::div_ceil};
    use nom::{bytes::complete::take, number::complete::be_u32, sequence::tuple, IResult};

    pub(crate) const HEADER: &[u8; 4] = b"IHDR";

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
        pub(crate) fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![0, 0, 0, 13];
            bytes.extend(HEADER);
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

    pub(crate) fn parse_data(chunk_data: &[u8]) -> IResult<&[u8], IHDRChunk> {
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
}

pub(crate) mod plte {
    use nom::IResult;

    pub(crate) const HEADER: &[u8; 4] = b"PLTE";

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub(crate) struct PLTEChunk<'a> {
        colors: &'a [u8],
    }
    impl PLTEChunk<'_> {
        pub(crate) fn get_color(&self, index: u8) -> Option<(u8, u8, u8)> {
            let index = index as usize;
            Some((
                *self.colors.get(index)?,
                *self.colors.get(index + 1)?,
                *self.colors.get(index + 2)?,
            ))
        }
    }

    pub(crate) fn parse_data(chunk_data: &[u8]) -> IResult<&[u8], PLTEChunk> {
        Ok((&chunk_data[0..0], PLTEChunk { colors: chunk_data }))
    }
}

mod phys {
    use nom::{
        number::complete::{be_u32, u8},
        sequence::tuple,
        IResult,
    };

    pub(crate) const HEADER: &[u8; 4] = b"pHYs";

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub(crate) struct pHYsChunk {
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

    pub(crate) fn parse_data(chunk_data: &[u8]) -> IResult<&[u8], pHYsChunk> {
        let (rest, (_x_axis_ppu, _y_axis_ppu, _unit_specifier)) =
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

pub(crate) mod idat {
    use crate::crc::calculate_crc;
    use nom::IResult;

    pub(crate) const HEADER: &[u8; 4] = b"IDAT";

    #[derive(Debug)]
    pub(crate) struct IDATChunk<T> {
        pub(crate) data: T,
    }
    impl<T> IDATChunk<T>
    where
        T: AsRef<[u8]>,
    {
        pub(crate) fn to_bytes(&self) -> Vec<u8> {
            let len = self.data.as_ref().len() as u32;
            let mut bytes = len.to_be_bytes().to_vec();
            bytes.extend(HEADER);
            bytes.extend(self.data.as_ref());
            let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
            bytes.extend(crc);
            bytes
        }
    }

    pub(crate) fn parse_data(chunk_data: &[u8]) -> IResult<&[u8], IDATChunk<&[u8]>> {
        Ok((&chunk_data[0..0], IDATChunk { data: chunk_data }))
    }
}

pub(crate) mod iend {
    use crate::crc::calculate_crc;

    pub(crate) const HEADER: &[u8; 4] = b"IEND";

    pub(crate) fn write_end() -> [u8; 12] {
        let mut data = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0];
        let crc = calculate_crc(data[4..8].iter().copied()).to_be_bytes();
        for (i, b) in crc.into_iter().enumerate() {
            data[i + 8] = b;
        }
        data
    }
}
