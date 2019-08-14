// Read and write vorbiscomment metadata

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

use lewton::header::CommentHeader;
use ogg::{PacketReader, PacketWriter, Packet};
use ogg::writing::PacketWriteEndInfo;
use std::io::{Cursor, Read, Seek};
use std::convert::TryInto;

/*
fn main() {
    let file_in = env::args().nth(1).expect("Please specify an input file.");
    let file_out = env::args().nth(2).expect("Please specify an output file.");
	println!("Opening files: {}, {}", file_in, file_out);

    //open files
	let mut f_in_disk = File::open(file_in).expect("Can't open file");
    let mut f_in_ram: Vec<u8> = vec![];
    let mut f_out_ram: Vec<u8> = vec![];

    std::io::copy(&mut f_in_disk, &mut f_in_ram).unwrap();
    
    let mut f_in = Cursor::new(&f_in_ram);
    //let mut f_out = Cursor::new(f_out_ram);
    let read_comments = read_comment_header(f_in);
    println!("Read comments: {:?}", read_comments);
    
    let mut f_in = Cursor::new(&f_in_ram);

    let mut vendor = "kaboink".to_string();
    let mut comment_list = Vec::with_capacity(2);
    comment_list.push((String::from("artist"), String::from("hejhopp")));
    comment_list.push((String::from("album"), String::from("tummetott")));
    let new_comment = CommentHeader {
		vendor,
		comment_list,
    };

	let mut f_out = replace_comment_header(f_in, new_comment);
    let mut f_out_disk = File::create(file_out).unwrap();
    f_out.seek(std::io::SeekFrom::Start(0)).unwrap();
    std::io::copy(&mut f_out, &mut f_out_disk).unwrap();
}
*/

pub fn make_comment_header(header: &CommentHeader) -> Vec<u8> {
    //Signature
    let start = [3u8, 118, 111, 114, 98, 105, 115];

    //Vendor number of bytes as u32
    let vendor = header.vendor.as_bytes();
    let vendor_len: u32 = vendor.len().try_into().unwrap();

    //end byte
    let end: u8 = 1;

    let mut new_packet: Vec<u8> = vec![];

    //write start
    new_packet.extend(start.iter().cloned());

    //write vendor
    new_packet.extend(vendor_len.to_le_bytes().iter().cloned());
    new_packet.extend(vendor.iter().cloned());

    //write number of comments
    let comment_nbr: u32 = header.comment_list.len().try_into().unwrap();
    new_packet.extend(comment_nbr.to_le_bytes().iter().cloned());

    let mut commentstrings: Vec<String> = vec![];
    //write each comment
    for comment in header.comment_list.iter() {
        commentstrings.push(format!("{}={}",comment.0, comment.1));
        //let commenstrings.last().as_bytes();
        let comment_len: u32 = commentstrings.last().unwrap().as_bytes().len().try_into().unwrap();
        new_packet.extend(comment_len.to_le_bytes().iter().cloned());
        new_packet.extend(commentstrings.last().unwrap().as_bytes().iter().cloned());
    }


    new_packet.push(end);
    //println!("{:?}",new_packet);
    new_packet
}

pub fn read_comment_header<T: Read + Seek>(f_in: T) -> CommentHeader {

    let mut reader = PacketReader::new(f_in);

	let packet :Packet = reader.read_packet_expected().unwrap();
    let stream_serial = packet.stream_serial();

	let mut packet: Packet = reader.read_packet_expected().unwrap();
    //println!("{:?}",packet.data);
	while packet.stream_serial() != stream_serial {
		packet = reader.read_packet_expected().unwrap();
        //println!("{:?}",packet.data);
	}
    let comment_hdr = lewton::header::read_header_comment(&packet.data).unwrap();
    //println!("{:?}", comment_hdr);
    comment_hdr
}

pub fn replace_comment_header<T: Read + Seek>(f_in: T, new_header: CommentHeader) -> Cursor<Vec<u8>> {

    let new_comment_data = make_comment_header(&new_header);

    let f_out_ram: Vec<u8> = vec![];
    let mut f_out = Cursor::new(f_out_ram);

    let mut reader = PacketReader::new(f_in);
	let mut writer = PacketWriter::new(&mut f_out);

    //println!("Read first packet");
	let packet :Packet = reader.read_packet_expected().unwrap();
    let stream_serial = packet.stream_serial();
	let absgp_page = packet.absgp_page();

    //println!("Write first packet");
	writer.write_packet(packet.data.into_boxed_slice(), 
                        stream_serial, 
                        PacketWriteEndInfo::NormalPacket, 
                        absgp_page).unwrap();

    //println!("Search for comment packet");
	let mut packet: Packet = reader.read_packet_expected().unwrap();
	while packet.stream_serial() != stream_serial {
        println!("Packet with other serial found");
        let stream_serial = packet.stream_serial();
	    let absgp_page = packet.absgp_page();
	    writer.write_packet(packet.data.into_boxed_slice(), 
                            stream_serial, 
                            PacketWriteEndInfo::NormalPacket, 
                            absgp_page).unwrap();
		packet = reader.read_packet_expected().unwrap();
	}

    //println!("Decode comments");
    let _comment_hdr = lewton::header::read_header_comment(&packet.data).unwrap();

    //println!("Write new cpmments");
    writer.write_packet(new_comment_data.into_boxed_slice(), 
                        packet.stream_serial(), 
                        PacketWriteEndInfo::NormalPacket, 
                        packet.absgp_page()).unwrap();

	loop {
		let rp = reader.read_packet();
        match rp {
            Ok(r) => {
		        match r {
			        Some(packet) => {
				        let inf = if packet.last_in_stream() {
					        PacketWriteEndInfo::EndStream
				        } else if packet.last_in_page() {
					        PacketWriteEndInfo::EndPage
				        } else {
					        PacketWriteEndInfo::NormalPacket
				        };
                        let lastpacket = packet.last_in_stream() && packet.last_in_page();
				        let stream_serial = packet.stream_serial();
				        let absgp_page = packet.absgp_page();
				        writer.write_packet(packet.data.into_boxed_slice(),
					        stream_serial,
					        inf,
					        absgp_page).unwrap();
                        if lastpacket {
                            break
                        }
			        },
			        // End of stream
			        None => break,
                }
            },
            Err(error) => {
                println!("Error reading packet: {:?}", error);
                break;
            },
		}
	}
    f_out
}

