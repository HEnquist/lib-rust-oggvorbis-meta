# Oggvorbismeta
[![Build Status](https://travis-ci.org/HEnquist/lib-rust-oggvorbis-meta.svg?branch=master)](https://travis-ci.org/HEnquist/lib-rust-oggvorbis-meta)

A simple Rust library to read and write VorbisComments tags in OggVorbis (*.ogg) audio files.

The basic reading and writing of Ogg files is handled by the Ogg crate: https://github.com/RustAudio/ogg

Reading out the existing comments in a file is done using the Lewton crate: https://github.com/RustAudio/lewton 

See the tag_file example for basic usage. It reads the tags in an input file, prints them and then replaces them with some sample tags. The result is written to a new file.

To run the example type:
```
cargo run --example tag_file path/to/infile.ogg path/to/outfile.ogg
```

## Tag names
A list of common tags can be found here: https://xiph.org/vorbis/doc/v-comment.html

## Usage
The workflow is to prepare a CommentHeader structure containing all the desired tags. This is then inserted in an ogg file by the "replace_comment_header" function. This will accept anything that implements the std::io::Read and std::io::Seek traits as input, and return a std::io::Cursor wrapping a buffer in ram.
```
let mut f_out = replace_comment_header(f_in, new_comments);
```
