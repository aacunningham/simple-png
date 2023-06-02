use super::ParseableChunk;
use nom::IResult;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct tRNSChunk<'a> {
    inner: &'a [u8],
}
impl<'a> tRNSChunk<'a> {
    pub(crate) fn as_greyscale(&self) -> u16 {
        u16::from_be_bytes(self.inner[0..2].try_into().unwrap())
    }
    pub(crate) fn as_truecolor(&self) -> (u16, u16, u16) {
        (
            u16::from_be_bytes(self.inner[0..2].try_into().unwrap()),
            u16::from_be_bytes(self.inner[2..4].try_into().unwrap()),
            u16::from_be_bytes(self.inner[4..6].try_into().unwrap()),
        )
    }
    pub(crate) fn as_palette(&self, index: u8) -> u8 {
        *self.inner.get(index as usize).unwrap_or(&255)
    }
}
impl<'a> ParseableChunk<'a> for tRNSChunk<'a> {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"tRNS";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&'a [u8], Self> {
        Ok((&chunk_data[0..0], tRNSChunk { inner: chunk_data }))
    }

    fn to_bytes(&self) -> Self::Output {
        unimplemented!()
    }
}
