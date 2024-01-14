use crate::enums::{ChannelMode, Granule, SideInformation};
use crate::util;

fn granule_extract(raw: &[u8], counter: &mut u16, mode: ChannelMode) -> Granule {
    match mode {
        ChannelMode::SingleChannel => Granule {
            part2_3_length: util::incremental_extract(raw, 12, counter),
            big_values: util::incremental_extract(raw, 9, counter),
            global_gain: util::incremental_extract(raw, 8, counter) as u16,
            scalefac_compress: util::incremental_extract(raw, 4, counter) as u8,
            windows_switching_flag: util::incremental_extract(raw, 1, counter) as u8,
            table_select: util::incremental_extract(raw, 20, counter),
            //block_type: incremental_extract(raw, 2,  counter),
            //mixed_block_flag: incremental_extract(raw, 1,  counter),
            // table_selection:  incremental_extract(raw, 10,  counter),
            //subblock_gain: incremental_extract(raw, 9,  counter),
            block_type: 0,
            mixed_block_flag: 0,
            table_select_two_regions: 0,
            subblock_gain: 0,
            region0_count: util::incremental_extract(raw, 4, counter) as u8,
            region1_count: util::incremental_extract(raw, 3, counter) as u8,
            preflag: util::incremental_extract(raw, 1, counter) as u8,
            scalefac_scale: util::incremental_extract(raw, 1, counter) as u8,
            count1table_select: util::incremental_extract(raw, 1, counter) as u8,
        },
        _ => Granule {
            part2_3_length: util::incremental_extract(raw, 24, counter),
            big_values: util::incremental_extract(raw, 18, counter),
            global_gain: util::incremental_extract(raw, 16, counter) as u16,
            scalefac_compress: util::incremental_extract(raw, 8, counter) as u8,
            windows_switching_flag: util::incremental_extract(raw, 2, counter) as u8,
            table_select: util::incremental_extract(raw, 30, counter),
            // block_type: incremental_extract(raw, 4,  counter),
            // mixed_block_flag: incremental_extract(raw, 2,  counter),
            // table_selection:  incremental_extract(raw, 20,  counter),
            // subblock_gain: incremental_extract(raw, 12,  counter),
            block_type: 0,
            mixed_block_flag: 0,
            table_select_two_regions: 0,
            subblock_gain: 0,
            region0_count: util::incremental_extract(raw, 8, counter) as u8,
            region1_count: util::incremental_extract(raw, 6, counter) as u8,
            preflag: util::incremental_extract(raw, 2, counter) as u8,
            scalefac_scale: util::incremental_extract(raw, 2, counter) as u8,
            count1table_select: util::incremental_extract(raw, 2, counter) as u8,
        }
    }
}

pub fn get_side_information_mono(raw: &[u8]) -> SideInformation {
    let mut counter: u16 = 0;
    SideInformation {
        main_data_begin: util::incremental_extract(raw, 9, &mut counter),
        private_bits: util::incremental_extract(raw, 5, &mut counter),
        scfsi: util::incremental_extract(raw, 4, &mut counter),
        granule0: granule_extract(raw, &mut counter, ChannelMode::SingleChannel),
        granule1: granule_extract(raw, &mut counter, ChannelMode::SingleChannel),
    }
}

pub fn get_side_information_stereo(raw: &[u8]) -> SideInformation {
    let mut counter: u16 = 0;
    SideInformation {
        main_data_begin: util::incremental_extract(raw, 9, &mut counter),
        private_bits: util::incremental_extract(raw, 3, &mut counter),
        scfsi: util::incremental_extract(raw, 8, &mut counter),
        granule0: granule_extract(raw, &mut counter, ChannelMode::Stereo),
        granule1: granule_extract(raw, &mut counter, ChannelMode::Stereo),
    }
}
