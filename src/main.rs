mod protocol;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::Listener::start("0.0.0.0:9000").await?;
    Ok(())
}

// async fn main() {
//     let payload = b"hello world";

//     let header = Header::new(
//         FrameType::Hello,
//         1,
//         payload.len() as u32,
//         42,
//         FrameFlags::ACK_REQUIRED,
//     );

//     let raw = header.encode();

//     println!("Encoded header: {:?} Payload: {:?}", raw, payload);
// }
