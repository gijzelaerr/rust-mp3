fn one_byte(raw: &[u8], pos: usize) -> u32 {
    raw[pos] as u32
}

fn two_byte(raw: &[u8], pos: usize) -> u32 {
    one_byte(raw, pos) | ((raw[pos - 1] as u32) << 8)
}

fn three_byte(raw: &[u8], pos: usize) -> u32 {
    two_byte(raw, pos) | ((raw[pos - 2] as u32) << 16)
}

fn four_byte(raw: &[u8], pos: usize) -> u32 {
    three_byte(raw, pos) | ((raw[pos - 3] as u32) << 24)
}


pub(crate) fn extract_bits(raw: &[u8], offset: u16, length: u16) -> u32 {
    let start_byte: u16 = offset / 8;
    let end_byte = (length + offset).div_ceil(8);
    let shift = (8 - ((length + offset) % 8)) % 8;
    let num_bytes = end_byte - start_byte;

    assert!(raw.len() >= end_byte as usize, "out of range, bits available {}, offset {}, length {}", raw.len() * 8, offset, length);

    let pos = (end_byte - 1) as usize;
    let shifted: u32 = match num_bytes {
        1 => one_byte(raw, pos) >> shift,
        2 => two_byte(raw, pos) >> shift,
        3 => three_byte(raw, pos) >> shift,
        4 => four_byte(raw, pos) >> shift,
        5 => four_byte(raw, pos) >> shift | ((raw[pos - 4] as u32) << 24),
        _ => panic!("not implemented"),
    };

    let mask = 2_u32.pow(length as u32) - 1;
    shifted & mask
}

#[cfg(test)]
mod tests {
    use crate::util::extract_bits;

    #[test]
    fn test_extract_bits() {
        for i in 0..32 {
            assert_eq!(extract_bits(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], i, 1), 1);
        }
        for i in 0..31 {
            assert_eq!(extract_bits(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], i, 2), 3);
        }

        for i in 0..30 {
            assert_eq!(extract_bits(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], i, 3), 7);
        }

        for i in 0..29 {
            assert_eq!(extract_bits(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], i, 4), 15);
        }
    }
}

pub fn incremental_extract(raw: &[u8], length: u16, counter: &mut u16) -> u32 {
    *counter += length;
    return extract_bits(raw, *counter - length, length);
}
