use super::{crc::calculate_crc, ParseableChunk};
use nom::IResult;

#[derive(Debug)]
pub(crate) struct IDATChunk<'a> {
    pub(crate) data: &'a [u8],
}
impl<'a> ParseableChunk<'a> for IDATChunk<'a> {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"IDAT";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&[u8], Self> {
        Ok((&chunk_data[0..0], IDATChunk { data: chunk_data }))
    }

    fn to_bytes(&self) -> Self::Output {
        let len = self.data.as_ref().len() as u32;
        let mut bytes = len.to_be_bytes().to_vec();
        bytes.extend(Self::HEADER);
        bytes.extend(self.data.as_ref());
        let crc = calculate_crc(bytes[4..].iter().copied()).to_be_bytes();
        bytes.extend(crc);
        bytes
    }
}
