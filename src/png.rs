use crate::decoder::PNGDecoder;

pub struct PNG;

impl PNG {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let decoder = PNGDecoder::new(bytes)?;
        let (decoder, _ihdr) = decoder.parse_ihdr()?;
        let mut decoded_data = vec![];
        let (mut decoder, idat) = decoder.parse_idat()?;
        decoded_data.append(&mut idat.decode_data());
        while let Ok((d, idat)) = decoder.parse_idat() {
            decoder = d;
            decoded_data.append(&mut idat.decode_data());
        }

        Ok(PNG)
    }
}
