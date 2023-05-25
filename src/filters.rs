use anyhow::anyhow;

trait FilterType {
    fn filter(x: u8, a: u8, b: u8, c: u8) -> u8;
    fn reconstruct(x: u8, a: u8, b: u8, c: u8) -> u8;
}

pub enum Filter {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}
impl Filter {
    #[allow(unused)]
    pub fn filter(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x.wrapping_sub(a),
            Filter::Up => x.wrapping_sub(b),
            Filter::Average => {
                let a = a as u16;
                let b = b as u16;
                x.wrapping_sub((a + b / 2) as u8)
            }
            Filter::Paeth => x.wrapping_sub(paeth_predictor(a, b, c)),
        }
    }

    pub fn reconstruct(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x.wrapping_add(a),
            Filter::Up => x.wrapping_add(b),
            Filter::Average => {
                let a = a as u16;
                let b = b as u16;
                x.wrapping_add((a + b / 2) as u8)
            }
            Filter::Paeth => x.wrapping_add(paeth_predictor(a, b, c)),
        }
    }
}
impl TryFrom<u8> for Filter {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Up),
            3 => Ok(Self::Average),
            4 => Ok(Self::Paeth),
            i => Err(anyhow!("Filter type {i} is unknown.")),
        }
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = i16::abs(p - a as i16);
    let pb = i16::abs(p - b as i16);
    let pc = i16::abs(p - c as i16);
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
