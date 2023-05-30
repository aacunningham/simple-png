use super::{crc::calculate_crc, ParseableChunk};

pub(crate) struct IENDChunk;
impl<'a> ParseableChunk<'a> for IENDChunk {
    type Output = [u8; 12];

    const HEADER: &'static [u8; 4] = b"IEND";

    fn from_bytes(chunk_data: &[u8]) -> nom::IResult<&[u8], Self> {
        Ok((chunk_data, Self))
    }

    fn to_bytes(&self) -> Self::Output {
        let mut data = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0];
        let crc = calculate_crc(data[4..8].iter().copied()).to_be_bytes();
        for (i, b) in crc.into_iter().enumerate() {
            data[i + 8] = b;
        }
        data
    }
}
