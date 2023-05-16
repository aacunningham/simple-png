use nom::{bytes::complete::tag, IResult};

pub(crate) fn parse_signature(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"\x89PNG\x0d\x0a\x1a\x0a")(input)
}
