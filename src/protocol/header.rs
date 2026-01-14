use std::convert::TryFrom;

use bitflags::bitflags;

pub const MBUS_MAGIC: [u8; 4] = *b"MBUS";
pub const MBUS_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 24;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FrameFlags: u8 {
        const ACK_REQUIRED = 0b0000_0001;
        const COMPRESSED   = 0b0000_0010;
        const URGENT       = 0b0000_0100;
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Hello = 0x0001,
    Subscribe = 0x0002,
    Unsubscribe = 0x0003,
    Publish = 0x0004,
    Event = 0x0005,
    Ack = 0x0006,
    Error = 0x0007,
    Ping = 0x0008,
    Pong = 0x0009,
}

impl TryFrom<u16> for FrameType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        Ok(match value {
            0x0001 => Self::Hello,
            0x0002 => Self::Subscribe,
            0x0003 => Self::Unsubscribe,
            0x0004 => Self::Publish,
            0x0005 => Self::Event,
            0x0006 => Self::Ack,
            0x0007 => Self::Error,
            0x0008 => Self::Ping,
            0x0009 => Self::Pong,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u8,
    pub flags: FrameFlags,
    pub frame_type: FrameType,
    pub msg_kind: u16,
    pub payload_len: u32,
    pub msg_id: u64,
}

impl Header {
    pub fn new(
        frame_type: FrameType,
        msg_kind: u16,
        payload_len: u32,
        msg_id: u64,
        flags: FrameFlags,
    ) -> Self {
        Self {
            magic: MBUS_MAGIC,
            version: MBUS_VERSION,
            flags,
            frame_type,
            msg_kind,
            payload_len,
            msg_id,
        }
    }

    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];

        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.version;
        buf[5] = self.flags.bits();

        buf[6..8].copy_from_slice(&(self.frame_type as u16).to_be_bytes());
        buf[8..10].copy_from_slice(&self.msg_kind.to_be_bytes());
        buf[10..12].copy_from_slice(&0u16.to_be_bytes()); // reserved

        buf[12..16].copy_from_slice(&self.payload_len.to_be_bytes());
        buf[16..24].copy_from_slice(&self.msg_id.to_be_bytes());

        buf
    }
}

impl TryFrom<[u8; HEADER_SIZE]> for Header {
    type Error = &'static str;

    fn try_from(buf: [u8; HEADER_SIZE]) -> Result<Self, Self::Error> {
        if buf[0..4] != MBUS_MAGIC {
            return Err("invalid magic");
        }

        let frame_type = FrameType::try_from(u16::from_be_bytes([buf[6], buf[7]]))
            .map_err(|_| "invalid frame type")?;

        Ok(Self {
            magic: MBUS_MAGIC,
            version: buf[4],
            flags: FrameFlags::from_bits_truncate(buf[5]),
            frame_type,
            msg_kind: u16::from_be_bytes([buf[8], buf[9]]),
            payload_len: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            msg_id: u64::from_be_bytes([
                buf[16], buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23],
            ]),
        })
    }
}
