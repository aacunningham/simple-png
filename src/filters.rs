use anyhow::anyhow;

trait FilterType {
    fn filter(x: u8, a: u8, b: u8, c: u8) -> u8;
    fn reconstruct(x: u8, a: u8, b: u8, c: u8) -> u8;
}

pub enum Filter {
    None,
    Sub,
}
impl Filter {
    #[allow(unused)]
    pub fn filter(&self, x: u8, a: u8, _b: u8, _c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x - a,
        }
    }

    pub fn reconstruct(&self, x: u8, a: u8, _b: u8, _c: u8) -> u8 {
        match self {
            Filter::None => x,
            Filter::Sub => x.overflowing_add(a).0,
        }
    }
}
impl TryFrom<u8> for Filter {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            i => Err(anyhow!("We don't support {i}")),
        }
    }
}
