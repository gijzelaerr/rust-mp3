# rust-mp3
To teach myself better rust skills i want to create a mp3 decoder from scratch.


https://en.wikipedia.org/wiki/MP3

## plan

### step 1 - Parse music.mp3

music.mp3 is a mp3 file with the following specs:
 * ID3 version 2.3.0
 * MPEG ADTS
 * layer III
 * v1
 * 224 kbps
 * 32 kHz
 * Stereo

The mp3 spec
* http://www.mp3-tech.org/programmer/docs/mp3_theory.pdf 
* https://www.diva-portal.org/smash/get/diva2:830195/FULLTEXT01.pdf
*  https://github.com/ejmahler/rust_dct

 
 * [X] read in file to memory
 * [X] parse id3v2.3.0 Header http://fileformats.archiveteam.org/wiki/ID3 
 * [X] parse mpeg Header http://mpgedit.org/mpgedit/mpeg_format/mpeghdr.htm
 * [X] decode mp3 frame for music.mp3
 * [X] get bit stream, find Header
 * [X] decode side information
 * [ ] decode scale factors
 * [ ] decode huffman data
 * [ ] requantize spectrum
 * [ ] reorder spectrum
 * [ ] joint stereo processing, if applicable
 * [ ] alias reduction
 * [ ] synthesize via imdct * overlap-add method
 * [ ] synthesize via polyphase filter bank
 * [ ] output pcm samples
 * [ ] write to wav

 ### step 2 - make more universe

* [ ] add support for other id3v Header
* [ ] add decoding support for other non-music.mp3 specs
* [ ] stream file
* [ ] play audio

### notes

In ADTS the layer seems to be set to 1, not 0 for music.mp3
