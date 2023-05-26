use crate::chunks::{
    ihdr::{ColorType, IHDRChunk},
    plte::PLTEChunk,
};
use anyhow::anyhow;
use nom::{
    bits::{bits, complete::take},
    combinator::map,
    error::Error,
    multi::count,
    sequence::tuple,
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pixel {
    pub(crate) red: u16,
    pub(crate) green: u16,
    pub(crate) blue: u16,
    pub(crate) alpha: u16,
}
impl Pixel {
    pub fn new(red: u16, green: u16, blue: u16, alpha: u16) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

struct IndexedPixel(u8);
impl IndexedPixel {
    fn to_pixel(&self, palette: &PLTEChunk) -> Result<Pixel, anyhow::Error> {
        let (red, green, blue) = palette
            .get_color(self.0)
            .ok_or(anyhow!("color could not be found in palette"))?;
        Ok(Pixel {
            red: red as u16,
            green: green as u16,
            blue: blue as u16,
            alpha: u16::MAX,
        })
    }
}

pub(crate) fn parse_pixels<'a, I: Iterator<Item = &'a [u8]>>(
    scanlines: I,
    header: &IHDRChunk,
    palette: Option<&PLTEChunk>,
) -> anyhow::Result<Vec<Pixel>> {
    let mut all_pixels = Vec::with_capacity((header.width * header.height) as usize);
    for scanline in scanlines {
        let pixels = match header.color_type {
            ColorType::Greyscale => {
                bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(count(
                    parse_greyscale(header.bit_depth),
                    header.width as usize,
                ))(scanline)
                .map_err(|e| e.to_owned())?
                .1
            }
            ColorType::IndexedColor => bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(count(
                parse_indexed_color(header.bit_depth),
                header.width as usize,
            ))(scanline)
            .map_err(|e| e.to_owned())?
            .1
            .into_iter()
            .map(|p| {
                palette
                    .ok_or(anyhow!("A pLTe chunk is needed for IndexedColor type"))
                    .and_then(|plte| p.to_pixel(plte))
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
            ColorType::GreyscaleWithAlpha => {
                bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(count(
                    parse_greyscale_with_alpha(header.bit_depth),
                    header.width as usize,
                ))(scanline)
                .map_err(|e| e.to_owned())?
                .1
            }
            ColorType::Truecolor => {
                bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(count(
                    parse_truecolor(header.bit_depth),
                    header.width as usize,
                ))(scanline)
                .map_err(|e| e.to_owned())?
                .1
            }
            ColorType::TruecolorWithAlpha => {
                bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(count(
                    parse_truecolor_with_alpha(header.bit_depth),
                    header.width as usize,
                ))(scanline)
                .map_err(|e| e.to_owned())?
                .1
            }
        };
        all_pixels.extend(pixels);
    }
    Ok(all_pixels)
}

fn parse_greyscale(bit_depth: u8) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, intensity) = take_scaled(bit_depth)((input, bit_offset))?;

        Ok((
            rest,
            Pixel {
                red: intensity,
                green: intensity,
                blue: intensity,
                alpha: u16::MAX,
            },
        ))
    }
}

fn parse_indexed_color(
    bit_depth: u8,
) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), IndexedPixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, pixel) = take(bit_depth)((input, bit_offset))?;
        Ok((rest, IndexedPixel(pixel)))
    }
}

fn parse_greyscale_with_alpha(
    bit_depth: u8,
) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, (intensity, alpha)) =
            tuple((take_scaled(bit_depth), take_scaled(bit_depth)))((input, bit_offset))?;

        Ok((
            rest,
            Pixel {
                red: intensity,
                green: intensity,
                blue: intensity,
                alpha,
            },
        ))
    }
}

fn parse_truecolor(bit_depth: u8) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, (red, green, blue)) = tuple((
            take_scaled(bit_depth),
            take_scaled(bit_depth),
            take_scaled(bit_depth),
        ))((input, bit_offset))?;

        Ok((
            rest,
            Pixel {
                red,
                green,
                blue,
                alpha: u16::MAX,
            },
        ))
    }
}

fn parse_truecolor_with_alpha(
    bit_depth: u8,
) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, (red, green, blue, alpha)) = tuple((
            take_scaled(bit_depth),
            take_scaled(bit_depth),
            take_scaled(bit_depth),
            take_scaled(bit_depth),
        ))((input, bit_offset))?;

        Ok((
            rest,
            Pixel {
                red,
                green,
                blue,
                alpha,
            },
        ))
    }
}

fn take_scaled<'a>(
    bit_depth: u8,
) -> impl FnMut((&'a [u8], usize)) -> IResult<(&'a [u8], usize), u16> {
    map(take(bit_depth), move |v: u16| {
        if bit_depth == 16 {
            v
        } else {
            v * (u16::MAX / (2u16.pow(bit_depth as u32) - 1))
        }
    })
}
