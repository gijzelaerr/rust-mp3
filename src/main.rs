mod id3;
mod mpeg;

use crc::{Crc, CRC_16_IBM_SDLC};
use std::env;

use crate::mpeg::{get_mpeg_audio_frame, check_mpeg_audio_frame, Protection};
use crate::id3::get_id3_frames;

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
    let header = id3::id3v2(&raw[0..10])?;
    println!("header size: {}", header.size);
    println!("experimental: {}", header.experimental);
    println!("unsynchronization: {}", header.unsynchronization);
    println!("extended: {}", header.extended);

    let (_frames, initial_offset) = get_id3_frames(&raw[..header.size+10], header)?;

    let mut offset = initial_offset;
    loop {
        if !(raw[offset] == 0xff && (raw[offset + 1] & 0b1110_0000) == 0b1110_0000) {
            println!("skipping byte {}", offset);
            offset += 1;
            continue;
        }
        // 	Frame sync (all bits set)
        println!("frame sync");
        let audio_frame_header = get_mpeg_audio_frame(&raw, offset);
        let derived_header_values = check_mpeg_audio_frame(&audio_frame_header);

        let frame_offset;
        if audio_frame_header.protected == Protection::CRC {
            frame_offset = offset + 6;
        } else {
            frame_offset = offset + 4;
        }

        let frame_end = offset + frame_offset + derived_header_values.frame_length_in_bytes as usize;
        let frame = &raw[offset + frame_offset..frame_end];
        if audio_frame_header.protected == Protection::CRC {
            pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
            let crc = ((raw[offset + 4] as u16) << 8) | raw[offset + 5] as u16;
            if X25.checksum(frame) != crc {
                return Err("checksum doesn't match".into());
            }
        }
        println!("WE HAVE A FRAME");

        if frame_end >= raw.len() {
            println!("fin!");
            break;
        } else {
            offset = frame_end;
        }

    }
    Ok(())
}

