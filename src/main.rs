pub(crate) mod mpeg;
pub(crate) mod util;
pub(crate) mod id3;

use crc::{Crc, CRC_16_IBM_SDLC};
use std::env;

use mpeg::{get_mpeg_audio_frame, check_mpeg_audio_frame, Protection};
use id3::get_id3_frames;

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
    //println!("header: {:#?}", header);

    let (_frames, post_header_offset) = get_id3_frames(&raw[..header.size + 10], header)?;
    println!("post header offset: {}", post_header_offset);

    let mut offset = post_header_offset;

    let mut firsts = 3;

    loop {
        if !(raw[offset] == 0xff && (raw[offset + 1] & 0b1110_0000) == 0b1110_0000) {
            println!("skipping byte {}", offset);
            offset += 1;
            continue;
        }
        // 	Frame sync (all bits set)
        let audio_frame_header = get_mpeg_audio_frame(&raw[offset..offset + 4]);
        let derived_header_values = check_mpeg_audio_frame(&audio_frame_header);

        if firsts > 0 {
            //println!("audio_frame_header {:#?}", audio_frame_header);
            //println!("derived_header_values {:#?}", derived_header_values);
        }

        0;
        let _frame_crc_offset;
        if audio_frame_header.protected == Protection::CRC {
            _frame_crc_offset = 2;
        } else {
            _frame_crc_offset = 0;
        }

        // so it turns out frame_length_in_bytes includes the header
        let frame_body_start = offset;
        let frame_body_end = frame_body_start + derived_header_values.frame_length_in_bytes as usize;
        let frame_body = &raw[frame_body_start..frame_body_end];
        if audio_frame_header.protected == Protection::CRC {
            pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
            let crc = ((raw[offset + 4] as u16) << 8) | raw[offset + 5] as u16;
            if X25.checksum(frame_body) != crc {
                return Err("checksum doesn't match".into());
            }
        }

        let side_information = match audio_frame_header.channel_mode {
            mpeg::ChannelMode::SingleChannel => mpeg::get_side_information_mono(&frame_body[mpeg::MPEG_FRAME_HEADER_LENGTH .. mpeg::MPEG_FRAME_HEADER_LENGTH + 17]),
            _ => mpeg::get_side_information_stereo(&frame_body[mpeg::MPEG_FRAME_HEADER_LENGTH .. mpeg::MPEG_FRAME_HEADER_LENGTH +32])
        };

        let main_data = match audio_frame_header.channel_mode {
            mpeg::ChannelMode::SingleChannel => &frame_body[mpeg::MPEG_FRAME_HEADER_LENGTH + 17 ..],
            _ => &frame_body[mpeg::MPEG_FRAME_HEADER_LENGTH + 32 ..]
        };

        println!("{:?}", main_data.len());

        if firsts > 0 {
            println!("side_information {:#?}", side_information);
        }
        if frame_body_end >= raw.len() {
            println!("fin!");
            return Ok(());
        }

        offset = frame_body_end;
        firsts -= 1;
    }
}

