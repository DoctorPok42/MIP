use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::protocol::{Header, HEADER_SIZE};

#[derive(Debug, Clone)]
pub struct Frame {
    pub header: Header,
    pub payload: Vec<u8>,
}

impl Frame {
    pub async fn read_from<R>(reader: &mut R) -> tokio::io::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_buf).await?;

        let header = Header::try_from(header_buf).map_err(|_| {
            tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, "invalid header")
        })?;

        let mut payload = vec![0u8; header.payload_len as usize];
        reader.read_exact(&mut payload).await?;

        Ok(Self { header, payload })
    }

    pub async fn write_to<W>(&self, writer: &mut W) -> tokio::io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let header_bytes = self.header.encode();
        writer.write_all(&header_bytes).await?;
        writer.write_all(&self.payload).await?;
        Ok(())
    }
}
