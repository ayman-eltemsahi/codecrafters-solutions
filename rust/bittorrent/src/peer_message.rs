use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl TryFrom<u8> for MessageType {
    type Error = &'static str;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == MessageType::Choke as u8 => Ok(MessageType::Choke),
            x if x == MessageType::Unchoke as u8 => Ok(MessageType::Unchoke),
            x if x == MessageType::Interested as u8 => Ok(MessageType::Interested),
            x if x == MessageType::NotInterested as u8 => Ok(MessageType::NotInterested),
            x if x == MessageType::Have as u8 => Ok(MessageType::Have),
            x if x == MessageType::Bitfield as u8 => Ok(MessageType::Bitfield),
            x if x == MessageType::Request as u8 => Ok(MessageType::Request),
            x if x == MessageType::Piece as u8 => Ok(MessageType::Piece),
            x if x == MessageType::Cancel as u8 => Ok(MessageType::Cancel),
            _ => Err("unkown message type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub id: MessageType,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    pub fn from_empty_payload(id: MessageType) -> PeerMessage {
        PeerMessage {
            id,
            payload: vec![0; 0],
        }
    }

    pub fn interested(piece: u32, index: u32, block_length: u32) -> PeerMessage {
        let mut payload: Vec<u8> = vec![];
        payload.extend_from_slice(&piece.to_be_bytes());
        payload.extend_from_slice(&index.to_be_bytes());
        payload.extend_from_slice(&block_length.to_be_bytes());

        PeerMessage {
            id: MessageType::Request,
            payload,
        }
    }

    pub async fn read(stream: &mut TcpStream) -> anyhow::Result<PeerMessage> {
        let mut length_buf: [u8; 4] = [0; 4];
        let size = stream.read_exact(&mut length_buf).await?;
        assert_eq!(size, 4);
        let length = u32::from_be_bytes(length_buf);

        let mut buf = vec![0; length as usize];
        let size = stream.read_exact(&mut buf).await?;
        assert_eq!(size, length as usize);

        let id = buf[0].try_into().unwrap();
        let payload: Vec<u8> = buf[1..].try_into().unwrap();

        Ok(PeerMessage { id, payload })
    }

    pub async fn write(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let len = 1 + self.payload.len() as u32;

        let mut bytes: Vec<u8> = Vec::with_capacity(4 + len as usize);
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.push(self.id.clone() as u8);
        bytes.extend(&self.payload);

        stream.write_all(&bytes).await?;
        stream.flush().await?;

        Ok(())
    }
}

impl TryFrom<&[u8]> for PeerMessage {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let length = u32::from_be_bytes(value[..4].try_into().unwrap());
        let id = value[4].try_into().unwrap();
        let payload: Vec<u8> = value[5..length as usize].try_into().unwrap();

        Ok(PeerMessage { id, payload })
    }
}
