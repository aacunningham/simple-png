use super::{crc::calculate_crc, ParseableChunk};
use nom::{
    number::complete::{be_u32, u8},
    sequence::tuple,
    IResult,
};

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
        let mut bytes: Vec<u8> = Vec::with_capacity(21);
        bytes.extend(&[0, 0, 0, 9]);
        bytes.extend(Self::HEADER);
        bytes.extend(&self._x_axis_ppu.to_be_bytes());
        bytes.extend(&self._y_axis_ppu.to_be_bytes());
        bytes.push(self._unit_specifier);
        bytes.extend(calculate_crc(bytes[4..].iter().copied()).to_be_bytes());
        bytes
    }
}
