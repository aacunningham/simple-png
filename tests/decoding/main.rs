use simple_png::{Pixel, PNG};

const FILE: &[u8] = include_bytes!("test-2.png");

macro_rules! png_suite {
    ($($file:ident),*) => {
        mod png_suite {
            use simple_png::PNG;

            $(
            #[test]
            fn $file() {
                let input = std::fs::read(concat!(
                    "tests/decoding/png-suite/",
                    stringify!($file),
                    ".png"
                ))
                .unwrap();
                println!("{input:?}");
                insta::assert_debug_snapshot!(PNG::decode(&input).unwrap());
            }
            )*
        }
    };
}
macro_rules! png_suite_fail {
    ($($file:ident),*) => {
        mod png_suite_failures {
            use simple_png::PNG;

            $(
            #[test]
            #[should_panic]
            fn $file() {
                let input = std::fs::read(concat!(
                    "tests/decoding/png-suite/",
                    stringify!($file),
                    ".png"
                ))
                .unwrap();
                PNG::decode(&input).unwrap();
            }
            )*
        }
    };
}

#[test]
fn test_decoding() {
    PNG::decode(FILE).unwrap();
}

#[test]
fn test_round_trip() {
    let pixels = vec![
        Pixel::new(0, 0, 0, u16::MAX),
        Pixel::new(u16::MAX, u16::MAX, u16::MAX, u16::MAX),
        Pixel::new(u16::MAX, u16::MAX, u16::MAX, u16::MAX),
        Pixel::new(0, 0, 0, u16::MAX),
    ];
    let data = PNG::new(2, 2, pixels.clone()).encode();
    let p = PNG::decode(&data).unwrap();
    assert_eq!(pixels, p.pixels);
}

// We shouldn't expect the compression we ran to be 1-to-1 with
// the file we received.
#[test]
#[should_panic]
fn test_round_trip_2() {
    let p = PNG::decode(FILE).unwrap();
    let data = PNG::new(p.header.height, p.header.width, &p.pixels).encode();
    assert_eq!(FILE, data);
}

png_suite!(
    basn0g01, basn0g02, basn0g04, basn0g08, basn0g16, basn2c08, basn2c16, basn3p01, basn3p02,
    basn3p04, basn3p08, basn4a08, basn4a16, basn6a08, basn6a16, bgan6a08, bgan6a16, bgbn4a08,
    bggn4a16, bgwn6a08, bgyn6a16, ccwn2c08, ccwn3p08, cdfn2c08, cdhn2c08, cdsn2c08, cdun2c08,
    ch1n3p04, ch2n3p08, cm0n0g04, cm7n0g04, cm9n0g04, cs3n2c16, cs3n3p08, cs5n2c08, cs5n3p08,
    cs8n2c08, cs8n3p08, ct0n0g04, ct1n0g04, cten0g04, ctfn0g04, ctgn0g04, cthn0g04, ctjn0g04,
    ctzn0g04, exif2c08, f00n0g08, f00n2c08, f01n0g08, f01n2c08, f02n0g08, f02n2c08, f03n0g08,
    f03n2c08, f04n0g08, f04n2c08, f99n0g04, g03n0g16, g03n2c08, g03n3p04, g04n0g16, g04n2c08,
    g04n3p04, g05n0g16, g05n2c08, g05n3p04, g07n0g16, g07n2c08, g07n3p04, g10n0g16, g10n2c08,
    g10n3p04, g25n0g16, g25n2c08, g25n3p04, oi1n0g16, oi1n2c16, oi2n0g16, oi2n2c16, oi4n0g16,
    oi4n2c16, oi9n0g16, oi9n2c16, pp0n2c16, pp0n6a08, ps1n0g08, ps1n2c16, ps2n0g08, ps2n2c16,
    s01i3p01, s01n3p01, s02i3p01, s02n3p01, s03i3p01, s03n3p01, s04i3p01, s04n3p01, s05n3p02,
    s06n3p02, s07n3p02, s08n3p02, s09n3p02, s32n3p04, s33n3p04, s34n3p04, s35n3p04, s36n3p04,
    s37n3p04, s38n3p04, s39n3p04, s40n3p04, tbbn0g04, tbbn2c16, tbbn3p08, tbgn2c16, tbgn3p08,
    tbrn2c08, tbwn0g16, tbwn3p08, tbyn3p08, tm3n3p02, tp0n0g08, tp0n2c08, tp0n3p08, tp1n3p08,
    z00n2c08, z03n2c08, z06n2c08, z09n2c08, basi0g01, basi0g02, basi0g04, basi0g08, basi0g16,
    basi2c08, basi2c16, basi3p01, basi3p02, basi3p04, basi3p08, basi4a08, basi4a16, basi6a08,
    basi6a16, bgai4a08, bgai4a16, s05i3p02, s06i3p02, s07i3p02, s08i3p02, s09i3p02, s32i3p04,
    s33i3p04, s34i3p04, s35i3p04, s36i3p04, s37i3p04, s38i3p04, s39i3p04, s40i3p04
);

png_suite_fail!(
    xc1n0g08, xc9n2c08, xcrn0g04, xcsn0g01, xd0n2c08, xd3n2c08, xd9n2c08, xdtn0g01, xhdn0g08,
    xlfn0g04, xs1n0g01, xs2n0g01, xs4n0g01, xs7n0g01
);
