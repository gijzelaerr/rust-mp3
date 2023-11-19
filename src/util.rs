fn one_byte(raw: &[u8], i: usize) -> u32 {
    raw[i] as u32
}

fn two_byte(raw: &[u8], i: usize) -> u32 {
    one_byte(raw, i) | ((raw[i - 1] as u32) << 8)
}

fn three_byte(raw: &[u8], i: usize) -> u32 {
    two_byte(raw, i) | ((raw[i - 2] as u32) << 16)
}

fn four_byte(raw: &[u8], i: usize) -> u32 {
    three_byte(raw, i) | ((raw[i - 3] as u32) << 24)
}

pub(crate) fn extract_bits(raw: &[u8], offset: u8, length: u8) -> u32 {
    let start_byte: u8 = offset / 8;
    let end_byte = (length + offset).div_ceil(8);
    let shift = (8 - (offset % 8)) % 8;
    let num_bytes = end_byte - start_byte;

    let i = end_byte as usize;
    let shifted: u32 = match num_bytes {
        1 => one_byte(raw, i) >> shift,
        2 => two_byte(raw, i) >> shift,
        3 => three_byte(raw, i) >> shift,
        4 => four_byte(raw, i) >> shift,
        5 => four_byte(raw, i) >> shift | ((raw[i - 4] as u32) << 24),
        _ => panic!("not implemented"),
    } as u32;

    return (shifted & (2_u32.pow(length as u32) - 1)) as u32;
}