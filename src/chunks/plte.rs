use super::ParseableChunk;
use nom::{bytes::complete::take, combinator::map, multi::count, IResult};

#[derive(Debug)]
pub(crate) struct Entry(pub u8, pub u8, pub u8);

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct PLTEChunk {
    colors: Vec<Entry>,
}
impl PLTEChunk {
    pub(crate) fn get_color(&self, index: u8) -> Option<&Entry> {
        self.colors.get(index as usize)
    }
}
impl<'a> ParseableChunk<'a> for PLTEChunk {
    type Output = Vec<u8>;

    const HEADER: &'static [u8; 4] = b"PLTE";

    fn from_bytes(chunk_data: &'a [u8]) -> IResult<&'a [u8], Self> {
        let entry_count = chunk_data.len() / 3;
        let (rest, entries) = count(
            map(take(3usize), |i: &[u8]| Entry(i[0], i[1], i[2])),
            entry_count,
        )(chunk_data)?;
        Ok((rest, PLTEChunk { colors: entries }))
    }

    fn to_bytes(&self) -> Self::Output {
        unimplemented!()
    }
}
