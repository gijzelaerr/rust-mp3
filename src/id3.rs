use std::str;

pub struct Id3Header {
    pub(crate) size: usize,
    pub(crate) unsynchronization: bool,
    pub(crate) extended: bool,
    pub(crate) experimental: bool,
}


pub struct Tag<'s> {
    id: &'s str,
    content: &'s str,
}

pub fn get_size(header: &[u8]) -> usize {
    let size1 = header[0];
    let size2 = header[1];
    let size3 = header[2];
    let size4 = header[3];

    if size1 & 128 + size2 & 128 + size3 & 128 + size4 & 128 != 0 {
        panic!("invalid size information")
    }

    let mut shifted = [0; 4];

    shifted[3] = (size3 << 7) | size4;
    shifted[2] = (size2 << 6) | size3 >> 1;
    shifted[1] = (size1 << 5) | size2 >> 2;
    shifted[0] = size1 >> 3;
    return u32::from_be_bytes(shifted) as usize;
}


fn id3v2_3(header: &[u8]) -> Id3Header {
    // https://id3.org/d3v2.3.0
    let flags = header[5];
    let unsynchronization = (flags & 128) == 128;
    let extended = (flags & 64) == 64;
    let experimental = (flags & 32) == 32;
    let should_be_zero = (flags & 31) == 0;

    if !should_be_zero {
        panic!("invalid Header");
    }

    let size = get_size(&header[6..10]);
    return Id3Header { size, unsynchronization, extended, experimental };
}


pub fn id3v2(header: &[u8]) -> Result<Id3Header, &'static str> {
    let major_version = header[3];
    let minor_version = header[4];
    println!("major version {}", major_version);
    println!("minor version {}", minor_version);

    match major_version {
        3 => Ok(id3v2_3(header)),
        _ => Err("Unknown ID3 major version"),
    }
}

pub fn get_id3_frame(data: &[u8], offset: usize) -> Result<(&str, &str, usize), Box<dyn std::error::Error>> {
    let frame_id = str::from_utf8(&data[offset..offset + 4])?;
    let frame_size = get_size(&data[offset + 4..offset + 8]);
    let _flags = &data[offset + 8..offset + 10];
    let frame_content = str::from_utf8(&data[offset + 10..offset + 10 + frame_size])?;
    return Ok((frame_id, frame_content, frame_size));
}


pub fn get_id3_frames<'s>(raw: &'s [u8], header: Id3Header) -> Result<(Vec<Tag<'s>>, usize), Box<dyn std::error::Error>>  {
    let mut offset = 10;
    let mut tags = Vec::new();
    loop {
        let (id, content, size) = get_id3_frame(&raw, offset)?;
        tags.push(Tag { id, content });
        offset += 10 + size;
        if offset >= header.size {
            return Ok((tags, offset));
        }
    }
}
