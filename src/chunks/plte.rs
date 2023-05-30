use super::ParseableChunk;
use nom::IResult;

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
impl<'a> ParseableChunk<'a> for PLTEChunk<'a> {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"PLTE";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&'a [u8], Self> {
        Ok((&chunk_data[0..0], PLTEChunk { colors: chunk_data }))
    }

    fn to_bytes(&self) -> Self::Output {
        unimplemented!()
    }
}
