use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::length_data,
    number::complete::be_u32,
    sequence::{terminated, tuple},
    IResult,
};

mod crc;
pub(crate) mod idat;
pub(crate) mod iend;
pub(crate) mod ihdr;
mod phys;
pub(crate) mod plte;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum Chunk<'a> {
    IHDR(ihdr::IHDRChunk),
    PLTE(plte::PLTEChunk<'a>),
    pHYs(phys::pHYsChunk),
    IDAT(idat::IDATChunk<'a>),
    IEND,
    Unknown(RawChunk<'a>),
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

fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk<'_>> {
    let (rest, (header, chunk_data)) = valid_chunk(input)?;
    match header {
        ihdr::IHDRChunk::HEADER => Ok((
            rest,
            Chunk::IHDR(ihdr::IHDRChunk::from_bytes(chunk_data)?.1),
        )),
        plte::PLTEChunk::HEADER => Ok((
            rest,
            Chunk::PLTE(plte::PLTEChunk::from_bytes(chunk_data)?.1),
        )),
        phys::pHYsChunk::HEADER => Ok((
            rest,
            Chunk::pHYs(phys::pHYsChunk::from_bytes(chunk_data)?.1),
        )),
        idat::IDATChunk::HEADER => Ok((
            rest,
            Chunk::IDAT(idat::IDATChunk::from_bytes(chunk_data)?.1),
        )),
        iend::IENDChunk::HEADER => Ok((rest, Chunk::IEND)),
        _ => Ok((
            rest,
            Chunk::Unknown(RawChunk {
                _chunk_type: header,
                _chunk_data: chunk_data,
            }),
        )),
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
    let crc = crc::calculate_crc(
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

pub(crate) trait ParseableChunk<'a>: Sized {
    type Output: AsRef<[u8]>;
    const HEADER: &'static [u8; 4];

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&'a [u8], Self>;
    fn to_bytes(&self) -> Self::Output;
}
