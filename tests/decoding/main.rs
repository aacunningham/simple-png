use simple_png::chunks;
use simple_png::parse_signature;

const FILE: &[u8] = include_bytes!("test.png");

#[test]
fn test_decoding() {
    let (rest, sig) = parse_signature(FILE).unwrap();
    println!("{:?}", sig);
    let mut width = 0;
    for chunk in chunks::iter_chunks(rest) {
        match chunk {
            chunks::Chunk::IDAT(idat_chunk) => {
                let data = idat_chunk.decode_data();
                let scanline_length = width * 4 + 1;
                println!("{:?}", idat_chunk.decode_data().len());
                for filter_type in data.iter().step_by(scanline_length) {
                    println!("Filter type: {:?}", filter_type);
                }
            }
            chunks::Chunk::IHDR(ihdr) => {
                println!("{:?}", ihdr);
                width = ihdr.width as usize;
            }
            a => println!("{:?}", a),
        }
    }
}
