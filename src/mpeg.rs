use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

// all in kbps
static BITRATE_INDEX_V1_L1: [i32; 15] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
static BITRATE_INDEX_V1_L2: [i32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
static BITRATE_INDEX_V1_L3: [i32; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
static BITRATE_INDEX_V2_L1: [i32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
static BITRATE_INDEX_V2_L2L3: [i32; 15] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

static SAMPLE_RATE_MPEG1: [i32; 3] = [44100, 48000, 32000];
static SAMPLE_RATE_MPEG2: [i32; 3] = [22050, 24000, 16000];
static SAMPLE_RATE_MPEG2_5: [i32; 3] = [11025, 12000, 8000];


#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub venum Layer {
    Reserved = 0b00,
    III = 0b01,
    II = 0b10,
    I = 0b11,
}


#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum MpegVersion {
    IiV = 0b00, // MPEG Version 2.5
    Reserved = 0b01,
    II = 0b10,  // MPEG Version 2 ISO/IEC 13818-3
    I = 0b11,   // MPEG Version 1 ISO/IEC 11172-3
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Protection {
    CRC = 0b0,   // Protected by CRC (16bit crc follows header)
    No = 0b1,    // no protection
}

pub struct MpegAudioFrameHeader {
    version: MpegVersion,
    layer: Layer,
    pub(crate) protected: u8,
    bitrate_index: u8,
    sampling_rate_frequency_index: u8,
    padding: u8,
    private: u8,
    channel_mode: u8,
    mode_extension: u8,
    copyright: u8,
    original: u8,
    emphasis: u8,
}

pub struct DerivedMpegValues {
    bitrate: i32,
    samplerate: i32,
    pub(crate) frame_length_in_bytes: i32
}


pub fn get_mpeg_audio_frame(raw: &[u8], offset: usize) -> MpegAudioFrameHeader {
    // AAAAAAAA AAABBCCD EEEEFFGH IIJJKLMM
    // we skip A since we already know it's all 1

    //B
    let version = raw[offset + 1] >> 3 & 0b11;

    //C
    let layer = raw[offset + 1] >> 1 & 0b11;

    //D
    let protected = raw[offset + 1] & 0b1;

    //E
    let bitrate_index = raw[offset + 2] >> 4;

    //F
    let sampling_rate_frequency_index = raw[offset + 2] >> 2 & 0b11;

    //G
    let padding = raw[offset + 2] >> 1 & 0b1;

    //H
    let private = raw[offset + 2] & 0b1;

    //I
    let channel_mode = raw[offset + 3] >> 6;

    //J
    let mode_extension = raw[offset + 3] >> 4 & 0b11;

    //K
    let copyright = raw[offset + 3] >> 3 & 0b1;

    //L
    let original = raw[offset + 3] >> 2 & 0b1;

    //M
    let emphasis = raw[offset + 3] & 0b11;

    return MpegAudioFrameHeader {
        version: MpegVersion::try_from(version).unwrap(),
        layer: Layer::try_from(layer).unwrap(),
        protected,
        bitrate_index,
        sampling_rate_frequency_index,
        padding,
        private,
        channel_mode,
        mode_extension,
        copyright,
        original,
        emphasis,
    };
}

pub(crate) fn check_mpeg_audio_frame(header: &MpegAudioFrameHeader) -> DerivedMpegValues {
    if !(header.version as u8 & 0b10 == 0b10) {
        panic!("unsupported MPEG Audio version ID {:?}", header.version);
    }

    if header.layer == Layer::Reserved {
        panic!("reserved layer");
    }

    if header.bitrate_index == 0xff || header.bitrate_index > 14 {
        panic!("bad bitrate {}", header.bitrate_index);
    }

    if header.sampling_rate_frequency_index > 2 {
        panic!("bad samplerate {}", header.sampling_rate_frequency_index);
    }


    let bitrate = match header.version {
        MpegVersion::I => {
            match header.layer {
                Layer::I => BITRATE_INDEX_V1_L1[header.bitrate_index as usize] * 1000,
                Layer::II => BITRATE_INDEX_V1_L2[header.bitrate_index as usize] * 1000,
                Layer::III => BITRATE_INDEX_V1_L3[header.bitrate_index as usize] * 1000,
                _ => panic!("bad layer"),
            }
        }
        MpegVersion::II => {
            match header.layer {
                Layer::I => BITRATE_INDEX_V2_L1[header.bitrate_index as usize] * 1000,
                Layer::II | Layer::III => BITRATE_INDEX_V2_L2L3[header.bitrate_index as usize] * 1000,
                _ => panic!("bad layer"),
            }
        }

        _ =>  panic!("bad mpeg version"),
    };

    let samplerate = match header.version {
        MpegVersion::I => SAMPLE_RATE_MPEG1[header.sampling_rate_frequency_index as usize],
        MpegVersion::II => SAMPLE_RATE_MPEG2[header.sampling_rate_frequency_index as usize],
        MpegVersion::IiV => SAMPLE_RATE_MPEG2_5[header.sampling_rate_frequency_index as usize],
        _ => panic!("bay sample rate"),
    };

    let frame_length_in_bytes = match header.layer {
        Layer::I => (12 * bitrate / samplerate + header.padding as i32) * 4,
        Layer::II | Layer::III => 144 * bitrate / samplerate + header.padding as i32,
        _ => panic!("bad frame length"),
    };
    println!("mpeg_version {:?}", header.version);
    println!("layer {:?}", header.layer);
    println!("protected {}", header.protected);
    println!("bitrate {}", header.bitrate_index);
    println!("samplerate {}", header.sampling_rate_frequency_index);
    println!("padding {}", header.padding);
    println!("frame_length_in_bytes {}", frame_length_in_bytes);

    return DerivedMpegValues {
        bitrate,
        samplerate,
        frame_length_in_bytes,
    }
}

