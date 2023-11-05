use serde::{Deserialize, Serialize};
use serde_bytes;
use std::str;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TorrentFileInfo {
    /// The name key maps to a UTF-8 encoded string which is the suggested
    /// name to save the file (or directory) as.
    /// It is purely advisory.
    pub name: String,

    /// the number of bytes in each piece the file is split into.
    /// For the purposes of transfer, files are split into fixed-size
    /// pieces which are all the same length except for possibly the last
    /// one which may be truncated.
    /// piece length is almost always a power of two, most commonly 2 18 = 256 K
    /// (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    #[serde(rename = "piece length")]
    pub piece_length: usize,

    /// The length of the file, in bytes.
    pub length: usize,

    /// a string whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20, each of which
    /// is the SHA1 hash of the piece at the corresponding index.
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TorrentFile {
    /// The URL of the tracker.
    pub announce: String,

    pub info: TorrentFileInfo,
}

impl TorrentFile {
    pub fn from_u8_vec(vec: Vec<u8>) -> TorrentFile {
        let torrent: TorrentFile = serde_bencode::from_bytes(&vec[..]).unwrap();

        if torrent.info.pieces.len() % 20 != 0 {
            panic!(
                "Expected pieces to have a length which is a multiple of 20 but found: {}",
                torrent.info.pieces.len()
            );
        }

        torrent
    }

    pub fn get_no_of_pieces(&self) -> usize {
        self.info.pieces.len() / 20
    }

    pub fn get_piece_hash(&self, piece_index: usize) -> String {
        let bytes = self.info.pieces.chunks_exact(20).nth(piece_index).unwrap();

        hex::encode(&bytes)
    }
}
