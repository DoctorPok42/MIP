mod protocol;

use protocol::{FrameFlags, FrameType, Header};

fn main() {
    let payload = b"hello world";

    let header = Header::new(
        FrameType::Hello,
        1,
        payload.len() as u32,
        42,
        FrameFlags::ACK_REQUIRED,
    );

    let raw = header.encode();

    println!("Encoded header: {:?}", raw);
}
