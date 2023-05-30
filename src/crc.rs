const CRC_TABLE: [u32; 256] = {
    let mut table = [0; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut i = 0;
        while i < 8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            i += 1;
        }
        table[n as usize] = c;
        n += 1;
    }
    table
};

fn update_crc<I: IntoIterator<Item = u8>>(crc: u32, data: I) -> u32 {
    let mut new_crc = crc;
    for b in data.into_iter() {
        let index = (new_crc ^ b as u32) & 0xff;
        new_crc = CRC_TABLE[index as usize] ^ (new_crc >> 8);
    }
    new_crc
}

pub(crate) fn calculate_crc<I: IntoIterator<Item = u8>>(data: I) -> u32 {
    update_crc(0xffffffff, data) ^ 0xffffffff
}
