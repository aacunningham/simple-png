use super::ParseableChunk;
use nom::{
    number::complete::{be_u32, u8},
    sequence::tuple,
    IResult,
};

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

impl<'a> ParseableChunk<'a> for pHYsChunk {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"pHYs";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&[u8], Self> {
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

    fn to_bytes(&self) -> Self::Output {
        unimplemented!()
    }
}
