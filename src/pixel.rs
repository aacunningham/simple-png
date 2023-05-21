use crate::chunks::plte::PLTEChunk;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl Pixel {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

pub struct IndexedPixel(u8);
impl IndexedPixel {
    pub fn to_pixel(&self, palette: PLTEChunk) -> Option<Pixel> {
        let (red, green, blue) = palette.get_color(self.0)?;
        Some(Pixel {
            red,
            green,
            blue,
            alpha: u8::MAX,
        })
    }
}
