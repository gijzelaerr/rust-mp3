use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;




fn id3v2_3(header: &[u8]) -> (u32, bool, bool, bool) {
    // https://id3.org/d3v2.3.0
    let flags = header[5];
    let unsynchronisation = (flags & 128) == 128;
    let extended = (flags & 64) == 64;
    let experimental = (flags & 32) == 32;
    let should_be_zero = (flags & 31) == 0;

    if !should_be_zero {
        panic!("invalid header");
    }
    println!("unsynchronisation: {}", unsynchronisation);
    println!("extended: {}", extended);
    println!("experimental: {}", experimental);

    let size1 = header[6];
    let size2 = header[7];
    let size3 = header[8];
    let size4 = header[9];

    if size1 & 128 + size2 & 128 + size3 & 128 + size4 & 128 != 0 {
        panic!("invalid size information")
    }

    let mut shifted = [0; 4];

    shifted[3] = (size3 << 7) | size4;
    shifted[2] = (size2 << 6) | size3 >> 1;
    shifted[1] = (size1 << 5) | size2 >> 2;
    shifted[0] = size1 >> 3;
    let size = u32::from_be_bytes(shifted);
    println!("size: {}", size);
    return (size, unsynchronisation, extended, experimental);

}


fn id3v2(header: &[u8], mut f: File) -> io::Result<()> {
    let major_version = header[3];
    let minor_version = header[4];
    println!("major version {}", major_version);
    println!("minor version {}", minor_version);

    if major_version == 3 {
        let (size, _unsynchronisation, _extended, _experimental) = id3v2_3(header);
        f.seek(SeekFrom::Start(10));
        let mut tag: Vec<u8> = Vec::new();
        f.take(size.into()).read_to_end(&mut tag);
        println!("{:?}", tag)
        
    }
    return Ok(());
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("usage: {} file.mp3", &args[0]);
    }

    let id3 = "ID3".as_bytes();
    let tag = "TAG".as_bytes();

    let mut f = File::open(&args[1])?;
    let mut header = [0; 10];

    f.seek(SeekFrom::Start(0))?;
    f.read_exact(&mut header)?;
    
    if &header[0 .. 3] == id3 {
        println!("We are id3v2");
        id3v2(&header, f);
    }

    //let mut tag_buffer = [0; 10];
    //f.seek(SeekFrom::End(-128))?;
    //f.read_exact(&mut tag_buffer)?;
    //if tag_buffer == tag {
    //    println!("We are ID3v1")
    //}

    Ok(())
}