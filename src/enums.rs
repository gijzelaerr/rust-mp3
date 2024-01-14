use num_enum::TryFromPrimitive;

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
    pub(crate) version: MpegVersion,
    pub(crate) layer: Layer,
    pub(crate) protected: Protection,
    pub(crate) bitrate_index: u8,
    pub(crate) sampling_rate_frequency_index: u8,
    pub(crate) padding: u8,
    pub(crate) private: u8,
    pub(crate) channel_mode: ChannelMode,
    pub(crate) mode_extension: u8,
    pub(crate) copyright: u8,
    pub(crate) original: u8,
    pub(crate) emphasis: u8,
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
    pub(crate) part2_3_length: u32,
    pub(crate) big_values: u32,
    pub(crate) global_gain: u16,
    pub(crate) scalefac_compress: u8,
    pub(crate) windows_switching_flag: u8,

    // normal block
    pub(crate) table_select: u32,
    pub(crate) region0_count: u8,
    pub(crate) region1_count: u8,

    // start, stop and short block
    pub(crate) block_type: u8,
    pub(crate) mixed_block_flag: u8,
    pub(crate) table_select_two_regions: u32,
    pub(crate) subblock_gain: u16,

    pub(crate) preflag: u8,
    pub(crate) scalefac_scale: u8,
    pub(crate) count1table_select: u8,
}
