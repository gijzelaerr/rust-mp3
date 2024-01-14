use std::convert::TryFrom;
use crate::enums::{ChannelMode, DerivedMpegValues, Layer, MpegAudioFrameHeader, MpegVersion, Protection};
use crate::util;

pub const MPEG_FRAME_HEADER_LENGTH: usize = 4; // bytes

static BITRATE_INDEX_V1_L1: [u32; 15] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
static BITRATE_INDEX_V1_L2: [u32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
static BITRATE_INDEX_V1_L3: [u32; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
static BITRATE_INDEX_V2_L1: [u32; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
static BITRATE_INDEX_V2_L2L3: [u32; 15] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

static SAMPLE_RATE_MPEG1: [u32; 3] = [44100, 48000, 32000];
static SAMPLE_RATE_MPEG2: [u32; 3] = [22050, 24000, 16000];
static SAMPLE_RATE_MPEG2_5: [u32; 3] = [11025, 12000, 8000];


pub fn get_mpeg_audio_frame(raw: &[u8]) -> MpegAudioFrameHeader {
    let mut counter: u16 = 0;
    // AAAAAAAA AAABBCCD EEEEFFGH IIJJKLMM
    let sync: u16 = util::incremental_extract(raw, 11, &mut counter) as u16;     // A
    let version: u8 = util::incremental_extract(raw, 2, &mut counter) as u8;   // B
    let layer: u8 = util::incremental_extract(raw, 2, &mut counter) as u8;      // C
    let protected: u8 = util::incremental_extract(raw, 1, &mut counter) as u8; // D
    let bitrate_index: u8 = util::incremental_extract(raw, 4, &mut counter) as u8; // E
    let sampling_rate_frequency_index: u8 = util::incremental_extract(raw, 2, &mut counter) as u8; // F
    let padding: u8 = util::incremental_extract(raw, 1, &mut counter) as u8; //G
    let private: u8 = util::incremental_extract(raw, 1, &mut counter) as u8; // H
    let channel_mode: u8 = util::incremental_extract(raw, 2, &mut counter) as u8; // I
    let mode_extension: u8 = util::incremental_extract(raw, 2, &mut counter) as u8; // J
    let copyright: u8 = util::incremental_extract(raw, 1, &mut counter) as u8; // K
    let original: u8 = util::incremental_extract(raw, 1, &mut counter) as u8;  // L
    let emphasis: u8 = util::incremental_extract(raw, 2, &mut counter) as u8; // M

    assert_eq!(sync, 0b11111111111, "bad sync");


    return MpegAudioFrameHeader {
        version: MpegVersion::try_from(version).unwrap(),
        layer: Layer::try_from(layer).unwrap(),
        protected: Protection::try_from(protected).unwrap(),
        bitrate_index,
        sampling_rate_frequency_index,
        padding,
        private,
        channel_mode: ChannelMode::try_from(channel_mode).unwrap(),
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

        _ => panic!("bad mpeg version"),
    };

    let samplerate = match header.version {
        MpegVersion::I => SAMPLE_RATE_MPEG1[header.sampling_rate_frequency_index as usize],
        MpegVersion::II => SAMPLE_RATE_MPEG2[header.sampling_rate_frequency_index as usize],
        MpegVersion::IiV => SAMPLE_RATE_MPEG2_5[header.sampling_rate_frequency_index as usize],
        _ => panic!("bay sample rate"),
    };

    let frame_length_in_bytes = match header.layer {
        Layer::I => (12 * bitrate / samplerate + header.padding as u32) * 4,
        Layer::II | Layer::III => 144 * bitrate / samplerate + header.padding as u32,
        _ => panic!("bad frame length"),
    };

    return DerivedMpegValues {
        bitrate,
        samplerate,
        frame_length_in_bytes,
    };
}
