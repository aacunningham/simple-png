use crate::{
    chunks::{
        ihdr::{ColorType, IHDRChunk},
        plte::{Entry, PLTEChunk},
        trns::tRNSChunk,
    },
    scanlines::ScanlineIterator,
};
use anyhow::anyhow;
use nom::{
    bits::{bits, complete::take},
    combinator::map,
    error::Error,
    multi::many0,
    sequence::tuple,
    IResult,
};

/// A Pixel of an image.
///
/// Currently the struct offers the highest possible precision PNG supports, RGB + alpha at 16 bit
/// depth. This should hopefully be replaced with an enum that captures the different variations
/// of pixel types.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pixel {
    /// The red channel of the Pixel. For greyscale images, this will equal both green and blue.
    pub red: u16,
    /// The green channel of the Pixel. For greyscale images, this will equal both red and blue.
    pub green: u16,
    /// The blue channel of the Pixel. For greyscale images, this will equal both green and red.
    pub blue: u16,
    /// The alpha channel of the Pixel. For images without transparency, this will be u16::MAX.
    pub alpha: u16,
}
impl Pixel {
    /// Construct a new Pixel from its parts.
    pub fn new(red: u16, green: u16, blue: u16, alpha: u16) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[derive(Debug)]
struct IndexedPixel(u8);
impl IndexedPixel {
    fn to_pixel(
        &self,
        palette: &PLTEChunk,
        transparency: Option<&tRNSChunk>,
    ) -> Result<Pixel, anyhow::Error> {
        let Entry(red, green, blue) = palette
            .get_color(self.0)
            .ok_or(anyhow!("color could not be found in palette"))?;
        Ok(Pixel {
            red: scale(*red as u16, 8),
            green: scale(*green as u16, 8),
            blue: scale(*blue as u16, 8),
            alpha: transparency.map_or(u16::MAX, |trns| scale(trns.as_palette(self.0).into(), 8)),
        })
    }
}

pub(crate) fn parse_scanline_pixels(
    scanline: &[u8],
    color_type: ColorType,
    bit_depth: u8,
    palette: Option<&PLTEChunk>,
    transparency: Option<&tRNSChunk>,
) -> anyhow::Result<Vec<Pixel>> {
    let pixels = match color_type {
        ColorType::Greyscale => {
            let transparent = transparency.map(tRNSChunk::as_greyscale);
            bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(many0(parse_greyscale(
                bit_depth,
                transparent,
            )))(&scanline[1..])
            .map_err(|e| e.to_owned())?
            .1
        }
        ColorType::IndexedColor => bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(many0(
            parse_indexed_color(bit_depth),
        ))(&scanline[1..])
        .map_err(|e| e.to_owned())?
        .1
        .into_iter()
        .map(|p| {
            // log::info!("{:?}", p);
            palette
                .ok_or(anyhow!("A pLTe chunk is needed for IndexedColor type"))
                .and_then(|plte| p.to_pixel(plte, transparency))
        })
        .collect::<anyhow::Result<Vec<_>>>()?,
        ColorType::GreyscaleWithAlpha => {
            bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(many0(parse_greyscale_with_alpha(
                bit_depth,
            )))(&scanline[1..])
            .map_err(|e| e.to_owned())?
            .1
        }
        ColorType::Truecolor => {
            let transparent_sample = transparency.map(tRNSChunk::as_truecolor);
            bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(many0(parse_truecolor(
                bit_depth,
                transparent_sample,
            )))(&scanline[1..])
            .map_err(|e| e.to_owned())?
            .1
        }
        ColorType::TruecolorWithAlpha => {
            bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(many0(parse_truecolor_with_alpha(
                bit_depth,
            )))(&scanline[1..])
            .map_err(|e| e.to_owned())?
            .1
        }
    };
    Ok(pixels)
}

pub(crate) fn parse_pixels<'a, S: ScanlineIterator<'a>>(
    iterator: S,
    header: &IHDRChunk,
    palette: Option<&PLTEChunk>,
    transparency: Option<&tRNSChunk>,
) -> anyhow::Result<Vec<Pixel>> {
    let mut total = vec![Pixel::default(); header.width as usize * header.height as usize];
    for (scanline, pixel_indices) in iterator {
        let pixels = parse_scanline_pixels(
            scanline,
            header.color_type,
            header.bit_depth,
            palette,
            transparency,
        )?;
        for (index, pixel) in pixel_indices.into_iter().zip(pixels.into_iter()) {
            total[index] = pixel;
        }
    }
    Ok(total)
}

fn parse_greyscale(
    bit_depth: u8,
    transparent: Option<u16>,
) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, intensity) = take(bit_depth)((input, bit_offset))?;
        let alpha = transparent.map_or(u16::MAX, |transparent| {
            if intensity == transparent {
                0
            } else {
                u16::MAX
            }
        });
        Ok((
            rest,
            Pixel {
                red: scale(intensity, bit_depth),
                green: scale(intensity, bit_depth),
                blue: scale(intensity, bit_depth),
                alpha,
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

fn parse_truecolor(
    bit_depth: u8,
    transparent: Option<(u16, u16, u16)>,
) -> impl Fn((&[u8], usize)) -> IResult<(&[u8], usize), Pixel> {
    move |(input, bit_offset): (&[u8], usize)| {
        let (rest, (red, green, blue)) =
            tuple((take(bit_depth), take(bit_depth), take(bit_depth)))((input, bit_offset))?;

        let alpha = transparent.map_or(u16::MAX, |samples| {
            if (red, green, blue) == samples {
                0
            } else {
                u16::MAX
            }
        });
        Ok((
            rest,
            Pixel {
                red: scale(red, bit_depth),
                green: scale(green, bit_depth),
                blue: scale(blue, bit_depth),
                alpha,
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
    map(take(bit_depth), move |v: u16| scale(v, bit_depth))
}

fn scale(value: u16, from_bit_depth: u8) -> u16 {
    if from_bit_depth == 16 {
        value
    } else {
        value * (u16::MAX / (2u16.pow(from_bit_depth as u32) - 1))
    }
}
