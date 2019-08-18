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
    let mut new_comment = CommentHeader::new();
    println!("Make new comment header");
    new_comment.vendor = "Ogg".to_string();
    new_comment.add_tag_single(&"artist".to_string(), &"Some Guy".to_string());
    new_comment.add_tag_single(&"artist".to_string(), &"Another Dude".to_string());
    new_comment.add_tag_single(&"album".to_string(), &"Greatest Hits".to_string());
    new_comment.add_tag_single(&"tracknumber".to_string(), &"3".to_string());
    new_comment.add_tag_single(&"title".to_string(), &"A very good song".to_string());
    new_comment.add_tag_single(&"date".to_string(), &"1997".to_string());


    println!("Insert new comments");
    let mut f_out = replace_comment_header(f_in, new_comment);

    println!("Save to disk");
    let mut f_out_disk = File::create(file_out).unwrap();
    f_out.seek(std::io::SeekFrom::Start(0)).unwrap();
    std::io::copy(&mut f_out, &mut f_out_disk).unwrap();
}