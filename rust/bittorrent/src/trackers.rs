use std::mem::size_of;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPeersRequest {
    pub announce_url: String,
    pub info_hash: [u8; 20],
    pub peer_id: String,
    pub port: i32,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: i8,
}

impl DiscoverPeersRequest {
    pub fn get_url(&self) -> String {
        let query = vec![
            ("peer_id", self.peer_id.to_string()),
            ("port", self.port.to_string()),
            ("uploaded", self.uploaded.to_string()),
            ("downloaded", self.downloaded.to_string()),
            ("left", self.left.to_string()),
            ("compact", self.compact.to_string()),
        ];

        let info_hash: String = self
            .info_hash
            .iter()
            .map(|v| format!("%{}", hex::encode(&[*v])))
            .collect();

        format!(
            "{}?{}&info_hash={}",
            &self.announce_url,
            serde_urlencoded::to_string(&query).unwrap(),
            info_hash
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPeersResponse {
    pub complete: usize,
    pub incomplete: usize,
    pub interval: usize,

    #[serde(rename = "min interval")]
    pub min_interval: usize,

    #[serde(with = "serde_bytes")]
    pub peers: Vec<u8>,
}

impl DiscoverPeersResponse {
    pub fn from_bencoded_bytes(
        bytes: &[u8],
    ) -> Result<DiscoverPeersResponse, serde_bencode::Error> {
        match serde_bencode::from_bytes(bytes) {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub fn parse_peers(&self) -> Vec<SocketAddr> {
        self.peers
            .chunks_exact(6)
            .map(|ch| {
                let ip = Ipv4Addr::new(ch[0], ch[1], ch[2], ch[3]);
                let port = u16::from_be_bytes([ch[4], ch[5]]);

                SocketAddr::new(IpAddr::V4(ip), port)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PeerHandshake {
    /// length of the protocol string (BitTorrent protocol) which is 19 (1 byte)
    pub length: u8,
    /// the string BitTorrent protocol (19 bytes)
    pub bittorrent: [u8; 19],
    /// eight reserved bytes, which are all set to zero (8 bytes)
    pub reserved: [u8; 8],
    /// sha1 infohash (20 bytes)
    pub info_hash: [u8; 20],

    /// peer id (20 bytes)
    pub peer_id: [u8; 20],
}

impl TryFrom<[u8; size_of::<PeerHandshake>()]> for PeerHandshake {
    type Error = ();

    fn try_from(value: [u8; size_of::<PeerHandshake>()]) -> Result<Self, Self::Error> {
        let mut i: usize = 0;
        let length = value[0] as u8;
        i += 1;

        let bittorrent: [u8; 19] = value[i..i + length as usize].try_into().unwrap();
        i += 19;

        let reserved: [u8; 8] = value[i..i + 8].try_into().unwrap();
        i += 8;

        let info_hash: [u8; 20] = value[i..i + 20].try_into().unwrap();
        i += 20;

        let peer_id: [u8; 20] = value[i..].try_into().unwrap();

        Ok(PeerHandshake {
            length,
            bittorrent,
            reserved,
            info_hash,
            peer_id,
        })
    }
}

impl PeerHandshake {
    pub fn from(info_hash: [u8; 20], peer_id: String) -> PeerHandshake {
        PeerHandshake {
            length: 19,
            bittorrent: "BitTorrent protocol".as_bytes().try_into().unwrap(),
            reserved: [0; 8],
            info_hash,
            peer_id: peer_id.as_bytes().try_into().unwrap(),
        }
    }

    pub async fn write_to_stream(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        stream.write_u8(self.length).await?;
        stream.write(&self.bittorrent).await?;
        stream.write(&self.reserved).await?;
        stream.write(&self.info_hash).await?;
        stream.write(&self.peer_id).await?;

        stream.flush().await?;

        Ok(())
    }
}
