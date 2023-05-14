use std::marker::PhantomData;

use anyhow::anyhow;
use nom::{bytes::complete::tag, IResult};

use crate::chunks::{idat, ihdr, parse_chunk, Chunk};

pub struct PNGDecoder<'a, State>(&'a [u8], PhantomData<State>);

pub struct Start;
pub struct Chunks;
pub struct IDAT;

impl<'a> PNGDecoder<'a, Start> {
    pub fn new(data: &'a [u8]) -> anyhow::Result<Self> {
        let (rest, _) = parse_signature(data)
            .map_err(|_| anyhow!("input doesn't start with expected signature"))?;
        Ok(Self(rest, PhantomData))
    }

    pub fn parse_ihdr(self) -> anyhow::Result<(PNGDecoder<'a, Chunks>, ihdr::IHDRChunk)> {
        let (rest, chunk) = ihdr::parse_chunk(self.0).map_err(|_| anyhow!("Junk"))?;
        Ok((PNGDecoder(rest, PhantomData), chunk))
    }
}

impl<'a, S> PNGDecoder<'a, S> {
    pub fn parse_idat(self) -> anyhow::Result<(PNGDecoder<'a, IDAT>, idat::IDATChunk<'a>)> {
        let mut data = self.0;
        while let Ok((rest, chunk)) = parse_chunk(data) {
            data = rest;
            match chunk {
                Chunk::IDAT(idat) => return Ok((PNGDecoder(data, PhantomData), idat)),
                _ => (),
            }
        }
        anyhow::bail!("Couldn't find an IDAT")
    }
}

fn parse_signature(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"\x89PNG\x0d\x0a\x1a\x0a")(input)
}
