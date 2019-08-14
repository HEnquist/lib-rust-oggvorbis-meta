// Read and write vorbiscomment metadata

extern crate oggvorbis_meta;
extern crate lewton;
extern crate byteorder;
extern crate ogg;

/*
fn main() {
	match run() {
		Ok(_) =>(),
		Err(err) => println!("Error: {}", err),
	}
}
*/

use std::env;
use lewton::header::CommentHeader;
use std::fs::File;
use std::io::{Cursor, Seek};
use oggvorbis_meta::{read_comment_header, replace_comment_header};


fn main() {
    let file_in = env::args().nth(1).expect("Please specify an input file.");
    let file_out = env::args().nth(2).expect("Please specify an output file.");
	println!("Opening files: {}, {}", file_in, file_out);

    //open files
	let mut f_in_disk = File::open(file_in).expect("Can't open file");
    let mut f_in_ram: Vec<u8> = vec![];

    println!("Copy input file to buffer");
    std::io::copy(&mut f_in_disk, &mut f_in_ram).unwrap();
    
    let f_in = Cursor::new(&f_in_ram);
    //let mut f_out = Cursor::new(f_out_ram);
    println!("Read comments from file");
    let read_comments = read_comment_header(f_in);
    println!("Existing comments: {:?}", read_comments);
    
    let f_in = Cursor::new(&f_in_ram);

    println!("Make new comment header");
    let vendor = "kaboink".to_string();
    let mut comment_list = Vec::with_capacity(2);
    comment_list.push((String::from("artist"), String::from("hejhopp")));
    comment_list.push((String::from("album"), String::from("tummetott")));
    let new_comment = CommentHeader {
		vendor,
		comment_list,
    };

    println!("Insert new comments");
	let mut f_out = replace_comment_header(f_in, new_comment);

    println!("Save to disk");
    let mut f_out_disk = File::create(file_out).unwrap();
    f_out.seek(std::io::SeekFrom::Start(0)).unwrap();
    std::io::copy(&mut f_out, &mut f_out_disk).unwrap();
}