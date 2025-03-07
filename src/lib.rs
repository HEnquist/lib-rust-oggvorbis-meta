//! Read and write vorbiscomment metadata

use ogg::writing::PacketWriteEndInfo;
use ogg::{Packet, PacketReader, PacketWriter};
use std::convert::TryInto;
use std::io::{Cursor, Read, Seek};
use thiserror::Error;

pub use lewton::header::CommentHeader;

pub trait VorbisComments {
    fn from(vendor: String, comment_list: Vec<(String, String)>) -> CommentHeader;
    fn new() -> Self;
    fn get_tag_names(&self) -> Vec<String>;
    fn get_tag_single(&self, tag: &str) -> Option<String>;
    fn get_tag_multi(&self, tag: &str) -> Vec<String>;
    fn clear_tag(&mut self, tag: &str);
    fn add_tag_single(&mut self, tag: impl Into<String>, value: impl Into<String>);
    fn add_tag_multi(&mut self, tag: impl Into<String>, values: &[String]);
    fn get_vendor(&self) -> String;
    fn set_vendor(&mut self, vend: impl Into<String>);
}

impl VorbisComments for CommentHeader {
    fn from(vendor: String, comment_list: Vec<(String, String)>) -> CommentHeader {
        CommentHeader {
            vendor,
            comment_list,
        }
    }

    fn new() -> CommentHeader {
        CommentHeader {
            vendor: String::new(),
            comment_list: Vec::new(),
        }
    }

    fn get_tag_names(&self) -> Vec<String> {
        let mut names = self
            .comment_list
            .iter()
            .map(|comment| comment.0.to_lowercase())
            .collect::<Vec<String>>();
        names.sort_unstable();
        names.dedup();
        names
    }

    fn get_tag_single(&self, tag: &str) -> Option<String> {
        let tags = self.get_tag_multi(tag);
        if tags.is_empty() {
            None
        } else {
            Some(tags[0].clone())
        }
    }

    fn get_tag_multi(&self, tag: &str) -> Vec<String> {
        self.comment_list
            .clone()
            .iter()
            .filter(|comment| comment.0.to_lowercase() == tag.to_lowercase())
            .map(|comment| comment.1.clone())
            .collect::<Vec<String>>()
    }

    fn clear_tag(&mut self, tag: &str) {
        self.comment_list
            .retain(|comment| comment.0.to_lowercase() != tag.to_lowercase());
    }

    fn add_tag_single(&mut self, tag: impl Into<String>, value: impl Into<String>) {
        self.comment_list
            .push((tag.into().to_lowercase(), value.into()));
    }

    fn add_tag_multi(&mut self, tag: impl Into<String>, values: &[String]) {
        let tag_string = tag.into();
        for value in values {
            self.comment_list
                .push((tag_string.clone().to_lowercase(), value.clone()));
        }
    }

    fn get_vendor(&self) -> String {
        self.vendor.to_string()
    }

    fn set_vendor(&mut self, vend: impl Into<String>) {
        self.vendor = vend.into();
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    OggReadError(#[from] ogg::OggReadError),
    #[error("{0}")]
    HeaderReadError(#[from] lewton::header::HeaderReadError),
    #[error("{0}")]
    WriteError(#[from] std::io::Error),
    #[error("{0}")]
    ParseError(#[from] std::num::TryFromIntError),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Create a new comment header packet.
/// # Arguments
/// * `header` - The comment header to write.
/// # Returns
/// * A `Vec<u8>` containing the comment header packet.
/// # Errors
/// * If any of the strings are too long to be converted to `u32`.
/// * If the comment header contains invalid characters.
pub fn make_comment_header(header: &CommentHeader) -> Result<Vec<u8>> {
    //Signature
    let start = [3u8, 118, 111, 114, 98, 105, 115];

    //Vendor number of bytes as u32
    let vendor = header.vendor.as_bytes();
    let vendor_len: u32 = vendor.len().try_into()?;

    //end byte
    let end: u8 = 1;

    let mut new_packet: Vec<u8> = vec![];

    //write start
    new_packet.extend(start.iter());

    //write vendor
    new_packet.extend(vendor_len.to_le_bytes().iter());
    new_packet.extend(vendor.iter());

    //write number of comments
    let comment_nbr: u32 = header.comment_list.len().try_into()?;
    new_packet.extend(comment_nbr.to_le_bytes().iter());

    //write each comment
    for comment in &header.comment_list {
        let val = format!("{}={}", comment.0, comment.1);
        let comment_len: u32 = val.as_bytes().len().try_into()?;
        new_packet.extend(comment_len.to_le_bytes().iter());
        new_packet.extend(val.as_bytes().iter());
    }
    new_packet.push(end);
    Ok(new_packet)
}

/// Read a comment header from an ogg file.
/// # Arguments
/// * `f_in` - A `Read + Seek` object
/// # Returns
/// * A `CommentHeader` object
/// # Errors
/// * If the file is not a valid ogg file.
/// * If the file does not contain a comment header.
pub fn read_comment_header<T: Read + Seek>(f_in: T) -> Result<CommentHeader> {
    let mut reader = PacketReader::new(f_in);

    let packet: Packet = reader.read_packet_expected()?;
    let stream_serial = packet.stream_serial();

    let mut packet: Packet = reader.read_packet_expected()?;
    while packet.stream_serial() != stream_serial {
        packet = reader.read_packet_expected()?;
    }
    Ok(lewton::header::read_header_comment(&packet.data)?)
}

/// Replace the comment header in an ogg file.
/// # Arguments
/// * `f_in` - A `Read + Seek` object.
/// * `new_header` - The new comment header.
/// # Returns
/// * A `Cursor<Vec<u8>>` containing the new ogg file.
/// # Errors
/// * If the file is not a valid ogg file.
/// * If the file does not contain a comment header.
/// * If the comment header contains invalid characters.
/// * If any of the strings are too long to be converted to `u32`.
/// * If there is an error writing the new file.
pub fn replace_comment_header<T: Read + Seek>(
    f_in: T,
    new_header: &CommentHeader,
) -> Result<Cursor<Vec<u8>>> {
    let new_comment_data = make_comment_header(new_header)?;

    let f_out_ram: Vec<u8> = vec![];
    let mut f_out = Cursor::new(f_out_ram);

    let mut reader = PacketReader::new(f_in);
    let mut writer = PacketWriter::new(&mut f_out);

    let mut header_done = false;
    while let Some(mut packet) = reader.read_packet()? {
        let inf = if packet.last_in_stream() {
            PacketWriteEndInfo::EndStream
        } else if packet.last_in_page() {
            PacketWriteEndInfo::EndPage
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        if !header_done {
            let comment_hdr = lewton::header::read_header_comment(&packet.data);
            if comment_hdr.is_ok() {
                packet.data.clone_from(&new_comment_data);
                header_done = true;
            }
        }
        let lastpacket = packet.last_in_stream() && packet.last_in_page();
        let stream_serial = packet.stream_serial();
        let absgp_page = packet.absgp_page();
        writer.write_packet(
            packet.data.into_boxed_slice(),
            stream_serial,
            inf,
            absgp_page,
        )?;
        if lastpacket {
            break;
        }
    }
    f_out.seek(std::io::SeekFrom::Start(0))?;
    Ok(f_out)
}
