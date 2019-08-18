// Read and write vorbiscomment metadata

extern crate oggvorbismeta;

use std::env;
use std::fs::File;
use std::io::{Cursor, Seek};
use oggvorbismeta::{read_comment_header, replace_comment_header, VorbisComments, CommentHeader};


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
    println!("Read comments from file");
    let read_comments = read_comment_header(f_in);
    
    let tag_names = read_comments.get_tag_names();
    println!("Existing tags: {:?}", tag_names);
    for tag in tag_names.iter() {
        println!("Existing tag: {:?}, {:?}", tag, read_comments.get_tag_multi(tag));
    }

    let f_in = Cursor::new(&f_in_ram);

    println!("Make new comment header");
    let vendor = "kaboink".to_string();
    let mut comment_list = Vec::new();
    comment_list.push((String::from("artist"), String::from("Some Guy")));
    comment_list.push((String::from("artist"), String::from("Another Dude")));
    comment_list.push((String::from("album"), String::from("Greatest Hits")));
    comment_list.push((String::from("tracknumber"), String::from("3")));
    comment_list.push((String::from("title"), String::from("A very good song")));
    comment_list.push((String::from("date"), String::from("1997")));
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