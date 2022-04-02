use std::env;
use std::str;

struct Header {
    size: u32,
    unsynchronization: bool,
    extended: bool,
    experimental: bool,
}

fn get_size(header: &[u8]) -> u32 {
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
    return u32::from_be_bytes(shifted);
}

fn id3v2_3(header: &[u8]) -> Header {
    // https://id3.org/d3v2.3.0
    let flags = header[5];
    let unsynchronization = (flags & 128) == 128;
    let extended = (flags & 64) == 64;
    let experimental = (flags & 32) == 32;
    let should_be_zero = (flags & 31) == 0;

    if !should_be_zero {
        panic!("invalid Header");
    }

    let size = get_size(&header[6 .. 10]);
    return Header { size, unsynchronization, extended, experimental };
}


fn id3v2(header: &[u8]) -> Result<Header, &'static str> {
    let major_version = header[3];
    let minor_version = header[4];
    println!("major version {}", major_version);
    println!("minor version {}", minor_version);

    match major_version {
        3 => Ok(id3v2_3(header)),
        _ => Err("Unknown ID3 major version"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(format!("usage: {} file.mp3", args[0]).into());
    }

    let file_path = &args[1];

    let id3 = "ID3".as_bytes();
    let _tag = "TAG".as_bytes();

    let content = std::fs::read(file_path)?;
    if &content[0..3] != id3 {
        return Err("unknown header version".into());
    }
    println!("We are id3v2");
    let header = id3v2(&content[0..10])?;
    println!("size: {}", header.size);
    println!("experimental: {}", header.experimental);
    println!("unsynchronization: {}", header.unsynchronization);
    println!("extended: {}", header.extended);
    let frame_id = str::from_utf8(&content[10..14])?;
    println!("frame id: {}", frame_id);
    let frame_size = get_size(&content[14..18]);
    let flags = &content[18..20];
    println!("frame_size: {}", frame_size);
    let frame_content =   str::from_utf8(&content[20..20+frame_size as usize])?;
    println!("frame content: {}", frame_content);

    Ok(())
}