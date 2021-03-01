use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        dbg!("usage: {} file.mp3", &args[0]);
    }
    let mut f = File::open(&args[1])?;
    //let mut buffer = [0; 4];
    
    // The frame header is constituted by the very first four bytes (32bits) in a frame.
    // The first eleven bits (or first twelve bits, see below about frame sync) of a frame
    // header are always set and they are called "frame sync". Therefore, you can search through
    // the file for the first occurence of frame sync (meaning that you have to find a byte with a
    // value of 255, and followed by a byte with its three (or four) most significant bits set).
    // Then you read the whole header and check if the values are correct. You will see in the following
    // table the exact meaning of each bit in the header, and which values may be checked for validity.
    // Each value that is specified as reserved, invalid, bad, or not allowed should indicate an invalid
    // header. Remember, this is not enough, frame sync can be easily (and very frequently) found in any
    // binary file. Also it is likely that MPEG file contains garbage on it's beginning which also may
    // contain false sync. Thus, you have to check two or more frames in a row to assure you are really
    // dealing with MPEG audio file.

    // read up to 10 bytes
    //f.read(&mut buffer)?;

    let mut data = Vec::new();
    f.read_to_end(&mut data)?;

    // and more! See the other methods for more details.
    Ok(())
}