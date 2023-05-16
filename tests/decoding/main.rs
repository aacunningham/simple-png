use simple_png::{encode, Pixel, PNG};

const FILE: &[u8] = include_bytes!("test-2.png");

#[test]
fn test_decoding() {
    PNG::from_bytes(FILE).unwrap();
}

#[test]
fn test_round_trip() {
    let pixels = vec![
        Pixel::new(0, 0, 0, 255),
        Pixel::new(255, 255, 255, 255),
        Pixel::new(255, 255, 255, 255),
        Pixel::new(0, 0, 0, 255),
    ];
    let data = encode(2, 2, &pixels);
    let p = PNG::from_bytes(&data).unwrap();
    assert_eq!(pixels, p.pixels);
}
