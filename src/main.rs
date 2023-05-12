use nom::{bytes::complete::tag, IResult};

mod chunks;

const FILE: &[u8] = include_bytes!("../test.png");

fn main() {
    let (rest, sig) = parse_signature(FILE).unwrap();
    println!("{:?}", sig);
    for chunk in chunks::iter_chunks(rest) {
        println!("{:?}", chunk);
    }
}

fn parse_signature(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"\x89PNG\x0d\x0a\x1a\x0a")(input)
}
