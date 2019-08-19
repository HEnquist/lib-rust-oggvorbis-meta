use oggvorbismeta;
use oggvorbismeta::{read_comment_header, replace_comment_header, VorbisComments, CommentHeader, make_comment_header};
use lewton;
use std::fs::File;

fn make_header() -> CommentHeader{
    let mut new_comment = CommentHeader::new();
    println!("Make new comment header");
    new_comment.set_vendor("Ogg");
    new_comment.add_tag_single("artist", "Some Guy");
    new_comment.add_tag_single("artist", "Another Dude");
    new_comment.add_tag_single("album", "Greatest Hits");
    new_comment.add_tag_single("tracknumber", "3");
    new_comment.add_tag_single("title", "A very good song");
    new_comment.add_tag_single("date", "1997");
    new_comment
}



#[test]
fn test_vendor() {
    let header = make_header();
    assert_eq!(header.get_vendor(), "Ogg".to_string());
}

#[test]
fn test_album() {
    let header = make_header();
    assert_eq!(header.get_tag_multi("album").len(), 1);
    assert_eq!(header.get_tag_multi("album")[0], "Greatest Hits".to_string());
    assert_eq!(header.get_tag_multi("ALBUM")[0], "Greatest Hits".to_string());
}

#[test]
fn test_artist() {
    let header = make_header();
    assert_eq!(header.get_tag_multi("artist").len(), 2);
    assert_eq!(header.get_tag_multi("artist")[0], "Some Guy".to_string());
    assert_eq!(header.get_tag_multi("artist")[1], "Another Dude".to_string());
}

#[test]
fn test_clear() {
    let mut header = make_header();
    assert_eq!(header.get_tag_multi("artist").len(), 2);
    header.clear_tag("artist");
    assert_eq!(header.get_tag_multi("artist").len(), 0);
}

#[test]
fn test_pack_unpack() {
    let header = make_header();
    let binary_header = make_comment_header(&header);
    let unpacked = lewton::header::read_header_comment(&binary_header).unwrap();
    assert_eq!(unpacked.get_tag_names().len(), 5);
    assert_eq!(unpacked.get_vendor(), "Ogg".to_string());
}

#[test]
fn test_read_from_file() {
    let f_in = File::open("tests/noise.ogg").expect("Can't open file");
    let read_comments = read_comment_header(f_in);
    assert_eq!(read_comments.get_tag_single("title"), "Noise".to_string());
}


#[test]
fn test_update_file() {

    let f_in = File::open("tests/noise.ogg").expect("Can't open file");
    let new_header = make_header();
    let f_out = replace_comment_header(f_in, new_header);
    let unpacked = read_comment_header(f_out);
    assert_eq!(unpacked.get_tag_names().len(), 5);
    assert_eq!(unpacked.get_vendor(), "Ogg".to_string());
}
