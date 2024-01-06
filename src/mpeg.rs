use std::cell::Cell;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
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


#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Layer {
    Reserved = 0b00,
    III = 0b01,
    II = 0b10,
    I = 0b11,
}


#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum MpegVersion {
    IiV = 0b00,
    // MPEG Version 2.5
    Reserved = 0b01,
    II = 0b10,
    // MPEG Version 2 ISO/IEC 13818-3
    I = 0b11,   // MPEG Version 1 ISO/IEC 11172-3
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Protection {
    CRC = 0b0,
    // Protected by CRC (16bit crc follows header)
    No = 0b1,    // no protection
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ChannelMode {
    Stereo = 0b00,
    JointStereo = 0b01,
    DualChannel = 0b10,
    SingleChannel = 0b11,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MpegAudioFrameHeader {
    version: MpegVersion,
    layer: Layer,
    pub(crate) protected: Protection,
    bitrate_index: u8,
    sampling_rate_frequency_index: u8,
    padding: u8,
    private: u8,
    pub(crate) channel_mode: ChannelMode,
    mode_extension: u8,
    copyright: u8,
    original: u8,
    emphasis: u8,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DerivedMpegValues {
    pub(crate) bitrate: u32,
    pub(crate) samplerate: u32,
    pub(crate) frame_length_in_bytes: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SideInformation {
    pub(crate) main_data_begin: u32,
    pub(crate) private_bits: u32,
    pub(crate) scfsi: u32,
    pub(crate) granule0: Granule,
    pub(crate) granule1: Granule,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Granule {
    part2_3_length: u32,
    big_values: u32,
    global_gain: u32,
    scalefac_compress: u32,
    windows_switching_flag: u32,
    block_type: u32,
    mixed_block_flag: u32,
    table_select: u32,
    subblock_gain: u32,
    region0_count: u32,
    region1_count: u32,
    preflag: u32,
    scalefac_scale: u32,
    count1table_select: u32,
}


pub fn get_mpeg_audio_frame(raw: &[u8]) -> MpegAudioFrameHeader {
    // AAAAAAAA AAABBCCD EEEEFFGH IIJJKLMM
    // we skip A since we already know it's all 1

    //B
    let version = raw[1] >> 3 & 0b11;

    //C
    let layer = raw[1] >> 1 & 0b11;

    //D
    let protected = raw[1] & 0b1;

    //E
    let bitrate_index = raw[2] >> 4;

    //F
    let sampling_rate_frequency_index = raw[2] >> 2 & 0b11;

    //G
    let padding = raw[2] >> 1 & 0b1;

    //H
    let private = raw[2] & 0b1;

    //I
    let channel_mode = raw[3] >> 6;

    //J
    let mode_extension = raw[3] >> 4 & 0b11;

    //K
    let copyright = raw[3] >> 3 & 0b1;

    //L
    let original = raw[3] >> 2 & 0b1;

    //M
    let emphasis = raw[3] & 0b11;


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

pub fn get_side_information_mono(raw: &[u8]) -> SideInformation {
    let counter = Cell::new(0);
    let incremental_extract = |length: u8| -> u32 {
        let new_counter = counter.get() + length;
        counter.set(new_counter);
        return util::extract_bits(raw, new_counter - length, length);
    };

    let granule_extract = || -> Granule {
        return Granule {
            part2_3_length: incremental_extract(12),
            big_values: incremental_extract(9),
            global_gain: incremental_extract(8),
            scalefac_compress: incremental_extract(4),
            windows_switching_flag: incremental_extract(1),
            block_type: incremental_extract(2),
            mixed_block_flag: incremental_extract(1),
            table_select: incremental_extract(20),
            subblock_gain: incremental_extract(9),
            region0_count: incremental_extract(4),
            region1_count: incremental_extract(3),
            preflag: incremental_extract(1),
            scalefac_scale: incremental_extract(1),
            count1table_select: incremental_extract(1),
        };
    };

    let main_data_begin = incremental_extract(9);
    let private_bits = incremental_extract(3);
    let scfsi = incremental_extract(4);
    let granule0 = granule_extract();
    let granule1 =  granule_extract();

    SideInformation {
        main_data_begin,
        private_bits,
        scfsi,
        granule0,
        granule1
    }
}

pub fn get_side_information_stereo(raw: &[u8]) -> SideInformation {
    let counter = Cell::new(0);
    let incremental_extract = |length: u8| -> u32 {
        let new_counter = counter.get() + length;
        counter.set(new_counter);
        return util::extract_bits(raw, new_counter - length, length);
    };


    let granule_extract = || -> Granule {
        return Granule {
            part2_3_length: incremental_extract(24),
            big_values: incremental_extract(18),
            global_gain: incremental_extract(16),
            scalefac_compress: incremental_extract(8),
            windows_switching_flag: incremental_extract(2),
            block_type: incremental_extract(4),
            mixed_block_flag: incremental_extract(2),
            table_select: incremental_extract(30),
            subblock_gain: incremental_extract(18),
            region0_count: incremental_extract(8),
            region1_count: incremental_extract(6),
            preflag: incremental_extract(2),
            scalefac_scale: incremental_extract(2),
            count1table_select: incremental_extract(2),
        };
    };

    let main_data_begin = incremental_extract(9);
    let private_bits = incremental_extract(5);
    let scfsi = incremental_extract(8);
    let granule0 = granule_extract();
    let granule1 =  granule_extract();

    SideInformation {
        main_data_begin,
        private_bits,
        scfsi,
        granule0,
        granule1
    }
}
