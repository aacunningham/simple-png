use once_cell::sync::Lazy;
use std::collections::HashMap;

static CRC_TABLE: Lazy<HashMap<u8, u32>> = Lazy::new(|| {
    let mut table = HashMap::new();
    for n in 0..=255 {
        let mut c = n as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        table.insert(n, c);
    }
    table
});

fn update_crc<I: IntoIterator<Item = u8>>(crc: u32, data: I) -> u32 {
    let mut new_crc = crc;
    for b in data.into_iter() {
        let index = (new_crc ^ b as u32) & 0xff;
        new_crc = CRC_TABLE.get(&(index as u8)).unwrap() ^ (new_crc >> 8);
    }
    new_crc
}

pub fn calculate_crc<I: IntoIterator<Item = u8>>(data: I) -> u32 {
    update_crc(0xffffffff, data) ^ 0xffffffff
}
