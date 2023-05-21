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

// We shouldn't expect the compression we ran to be 1-to-1 with
// the file we received.
#[test]
#[should_panic]
fn test_round_trip_2() {
    let p = PNG::from_bytes(FILE).unwrap();
    let data = encode(p.header.height, p.header.width, &p.pixels);
    assert_eq!(FILE, data);
}
