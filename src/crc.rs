use once_cell::sync::Lazy;

static CRC_TABLE: Lazy<Vec<u32>> = Lazy::new(|| {
    let mut table = Vec::with_capacity(u8::MAX as usize + 1);
    for n in u8::MIN..=u8::MAX {
        let mut c = n as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        table.push(c);
    }
    table
});

fn update_crc<I: IntoIterator<Item = u8>>(crc: u32, data: I) -> u32 {
    let mut new_crc = crc;
    for b in data.into_iter() {
        let index = (new_crc ^ b as u32) & 0xff;
        new_crc = CRC_TABLE[index as usize] ^ (new_crc >> 8);
    }
    new_crc
}

pub fn calculate_crc<I: IntoIterator<Item = u8>>(data: I) -> u32 {
    update_crc(0xffffffff, data) ^ 0xffffffff
}
