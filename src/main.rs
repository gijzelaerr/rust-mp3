use std::convert::{TryInto};
use crc::{Crc, CRC_16_IBM_SDLC};
use std::env;
use std::str;

struct Header {
    size: usize,
    unsynchronization: bool,
    extended: bool,
    experimental: bool,
}

// all in kbps
static BITRATE_INDEX_V1_L1: [i32; 15] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
static BITRATE_INDEX_V1_L2: [i32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
static BITRATE_INDEX_V1_L3: [i32; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
static BITRATE_INDEX_V2_L1: [i32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
static BITRATE_INDEX_V2_L2L3: [i32; 15] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

static SAMPLE_RATE_MPEG1: [i32; 3] = [44100, 48000, 32000];
static SAMPLE_RATE_MPEG2: [i32; 3] = [22050, 24000, 16000];
static SAMPLE_RATE_MPEG2_5: [i32; 3] = [11025, 12000, 8000];


macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

back_to_enum! {
enum Layer {
    Reserved = 0b00,
    III = 0b01,
    II = 0b10,
    I = 0b11,
}
    }

back_to_enum! {
enum Mpeg {
    IiV = 0b00, // MPEG Version 2.5
    Reserved = 0b01,
    II = 0b10, // ISO/IEC 13818-3
    I = 0b11, // ISO/IEC 11172-3
}
}



fn get_size(header: &[u8]) -> usize {
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

    let size = get_size(&header[6..10]);
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

fn get_frame(data: &[u8], offset: usize) -> Result<(&str, &str, usize), Box<dyn std::error::Error>> {
    let frame_id = str::from_utf8(&data[offset..offset + 4])?;
    let frame_size = get_size(&data[offset + 4..offset + 8]);
    let _flags = &data[offset + 8..offset + 10];
    let frame_content = str::from_utf8(&data[offset + 10..offset + 10 + frame_size])?;
    return Ok((frame_id, frame_content, frame_size));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(format!("usage: {} file.mp3", args[0]).into());
    }

    let file_path = &args[1];

    let id3 = "ID3".as_bytes();
    let _tag = "TAG".as_bytes();

    let raw = std::fs::read(file_path)?;
    if &raw[0..3] != id3 {
        return Err("unknown header version".into());
    }
    println!("We are id3v2");
    let header = id3v2(&raw[0..10])?;
    println!("header size: {}", header.size);
    println!("experimental: {}", header.experimental);
    println!("unsynchronization: {}", header.unsynchronization);
    println!("extended: {}", header.extended);

    let mut offset = 10;
    loop {
        let (id, content, size) = get_frame(&raw, offset)?;
        println!("id: {}, content: {}, size: {}, offset: {}", id, content, size, offset);
        offset += 10 + size;
        if offset >= header.size {
            break;
        }
    }

    for i in header.size + 10 .. raw.len() {
        if raw[i] == 0xff && (raw[i + 1] & 0b1110_0000) == 0b1110_0000 {
            // 	Frame sync (all bits set)
            let mpeg_version = raw[i + 1] >> 3 & 0b11;
            let layer = raw[i + 1] >> 1 & 0b11;
            let protected = raw[i + 1] & 0b1 != 0;

            let bitrate_raw = raw[i + 2] >> 4;
            let samplerate_raw = raw[i + 2] >> 2 & 0b11;
            let padding = raw[i + 2] >> 1 & 0b1;
            let _private = raw[i + 2] & 0b1;

            let _mode = raw[i + 3] >> 6;
            let _mode_extension = raw[i + 3] >> 4 & 0b11;

            let _copy = raw[i + 3] >> 3 & 0b1;
            let _home = raw[i + 3] >> 2 & 0b1;
            let _emphasis = raw[i + 3] & 0b11;

            if !mpeg_version & 0b10 == 0b10 {
                println!("unsupported MPEG Audio version ID {}", mpeg_version);
                continue;
            }

            if layer == Layer::Reserved as u8 {
                println!("reserved layer");
                continue;
            }

            if bitrate_raw == 0xff || bitrate_raw > 14 {
                println!("bad bitrate {}", bitrate_raw);
                continue;
            }

            if samplerate_raw > 2 {
                println!("bad samplerate {}", samplerate_raw);
                continue;
            }


            let bitrate = match mpeg_version.try_into() {
                Ok(Mpeg::I) => {
                    match layer.try_into() {
                        Ok(Layer::I) => BITRATE_INDEX_V1_L1[bitrate_raw as usize] * 1000,
                        Ok(Layer::II) => BITRATE_INDEX_V1_L2[bitrate_raw as usize] * 1000,
                        Ok(Layer::III) => BITRATE_INDEX_V1_L3[bitrate_raw as usize] * 1000,
                        Err(_) | Ok(Layer::Reserved) => continue,
                    }
                }
                Ok(Mpeg::II) => {
                    match layer.try_into() {
                        Ok(Layer::I) => BITRATE_INDEX_V2_L1[bitrate_raw as usize] * 1000,
                        Ok(Layer::II | Layer::III) => BITRATE_INDEX_V2_L2L3[bitrate_raw as usize] * 1000,
                        Err(_) | Ok(Layer::Reserved) => continue,
                    }
                }
                Err(_) | Ok(_) => continue,
            };

            let samplerate = match mpeg_version.try_into() {
                Ok(Mpeg::I) => SAMPLE_RATE_MPEG1[samplerate_raw as usize],
                Ok(Mpeg::II) => SAMPLE_RATE_MPEG2[samplerate_raw as usize],
                Ok(Mpeg::IiV) => SAMPLE_RATE_MPEG2_5[samplerate_raw as usize],
                Err(_) | Ok(_) => continue,
            };

            let frame_length_in_bytes = match layer.try_into() {
                Ok(Layer::I) => (12 * bitrate / samplerate + padding as i32) * 4,
                Ok(Layer::II | Layer::III) => 144 * bitrate / samplerate + padding as i32,
                Err(_) | Ok(Layer::Reserved) => continue,
            };

            //println!("mpeg_version {}", mpeg_version);
            //println!("layer {}", layer);
            //println!("protected {}", protected);
            //println!("bitrate {}", bitrate);
            //println!("samplerate {}", samplerate);
            //println!("padding {}", padding);
            //println!("frame_length_in_bytes {}", frame_length_in_bytes);

            let frame_offset;
            if protected {
                let crc = ((raw[i + 4] as u16) << 8) | raw[i + 5] as u16;
                pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
                frame_offset = i + 6;
                let frame = &raw[frame_offset..frame_offset + frame_length_in_bytes as usize];
                if X25.checksum(frame) != crc {
                    continue
                } else {
                    println!("WE HAVE A FRAME");
                }
            } else {
                frame_offset = i + 4;
            }
        }
    }
    Ok(())
}