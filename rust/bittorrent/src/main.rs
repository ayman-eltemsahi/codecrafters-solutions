mod bencode;
mod cmd_args;
mod hash;
mod peer_message;
mod torrent;
mod trackers;

use std::fs;
use std::mem::size_of;
use std::net::SocketAddr;

use clap::Parser;
use cmd_args::{Args, Command};
use hash::b_sha1;
use peer_message::MessageType;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use torrent::TorrentFile;
use trackers::PeerHandshake;

use crate::peer_message::PeerMessage;
use crate::trackers::{DiscoverPeersRequest, DiscoverPeersResponse};
use crate::{bencode::decode_bencoded_value, hash::hex_sha1};

const MAX_BLOCK_SIZE: usize = 1 << 14;

async fn get_peers(torrent: &TorrentFile) -> anyhow::Result<Vec<SocketAddr>> {
    let encoded_info = serde_bencode::to_bytes(&torrent.info).unwrap();

    let req = DiscoverPeersRequest {
        announce_url: torrent.announce.to_string(),
        info_hash: b_sha1(&encoded_info).try_into().unwrap(),
        peer_id: "00112233445566778899".to_string(),
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: torrent.info.length,
        compact: 1,
    };

    let bytes = reqwest::get(&req.get_url()).await?.bytes().await?;
    let peers_response = DiscoverPeersResponse::from_bencoded_bytes(&bytes[..]).unwrap();

    Ok(peers_response.parse_peers())
}

async fn download_piece(torrent: &TorrentFile, piece_index: usize) -> anyhow::Result<Vec<u8>> {
    let peers = get_peers(&torrent).await?;

    let encoded_info = serde_bencode::to_bytes(&torrent.info).unwrap();
    let info_hash = b_sha1(&encoded_info);

    let handshake = PeerHandshake::from(
        info_hash.try_into().unwrap(),
        "00112233445566778899".to_string(),
    );

    let mut stream = TcpStream::connect(peers.first().unwrap()).await?;

    handshake.write_to_stream(&mut stream).await?;

    let mut buf = [0; size_of::<PeerHandshake>()];
    stream.read_exact(&mut buf).await?;

    let handshake_response: PeerHandshake = buf.try_into().unwrap();

    println!("Peer ID: {}", hex::encode(handshake_response.peer_id));

    let bitfield = PeerMessage::read(&mut stream).await?;
    assert_eq!(bitfield.id, MessageType::Bitfield);

    PeerMessage::from_empty_payload(MessageType::Interested)
        .write(&mut stream)
        .await?;

    let unchoke = PeerMessage::read(&mut stream).await?;
    assert_eq!(unchoke.id, MessageType::Unchoke);

    let piece_length = torrent.info.piece_length;

    let start_index = piece_length * piece_index;
    let end_index = std::cmp::min(start_index + piece_length, torrent.info.length);

    let mut data: Vec<u8> = vec![];
    let mut cur_index = start_index;
    while cur_index < end_index {
        let block_length = std::cmp::min(MAX_BLOCK_SIZE, end_index - cur_index);

        PeerMessage::interested(
            piece_index as u32,
            (cur_index - start_index) as u32,
            block_length as u32,
        )
        .write(&mut stream)
        .await?;

        let piece = PeerMessage::read(&mut stream).await?;
        assert_eq!(piece.id, MessageType::Piece);

        let _res_index = i32::from_be_bytes(
            piece.payload[..4]
                .try_into()
                .expect("Could not read result index"),
        );
        let _res_begin = i32::from_be_bytes(
            piece.payload[4..8]
                .try_into()
                .expect("Could not read result begin"),
        );

        data.extend_from_slice(&piece.payload[8..]);
        cur_index += block_length;
    }

    let hash = hex_sha1(&data);
    assert_eq!(hash, torrent.get_piece_hash(piece_index));

    Ok(data)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            let decoded_value = decode_bencoded_value(&value);
            println!("{}", decoded_value.to_string());
        }

        Command::Info { filename } => {
            let contents = fs::read(filename).expect("Could not read the torrent file");
            let torrent = TorrentFile::from_u8_vec(contents);
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);

            let encoded_info = serde_bencode::to_bytes(&torrent.info)
                .expect("Could not encode torrent info to bytes");
            println!("Info Hash: {}", hex_sha1(&encoded_info));

            println!("Piece Length: {}", torrent.info.piece_length);
            println!("Piece Hashes: ");
            torrent.info.pieces.chunks_exact(20).for_each(|ch| {
                println!("{}", hex::encode(ch));
            })
        }

        Command::Peers { filename } => {
            let contents = fs::read(&filename).expect("Could not read the torrent file");

            get_peers(&TorrentFile::from_u8_vec(contents))
                .await?
                .iter()
                .for_each(|sock| println!("{}", sock.to_string()));
        }

        Command::Handshake { filename, peer } => {
            let sock: SocketAddr = peer.parse().expect("Could not parse peer addr");
            let contents = fs::read(&filename).expect("Could not read the torrent file");
            let torrent = TorrentFile::from_u8_vec(contents);

            let encoded_info = serde_bencode::to_bytes(&torrent.info).unwrap();
            let info_hash = b_sha1(&encoded_info);

            let handshake = PeerHandshake::from(
                info_hash.try_into().unwrap(),
                "00112233445566778899".to_string(),
            );

            let mut stream = TcpStream::connect(sock).await?;

            handshake.write_to_stream(&mut stream).await?;

            let mut buf = [0; size_of::<PeerHandshake>()];
            stream.read_exact(&mut buf).await?;

            let handshake_response: PeerHandshake = buf.try_into().unwrap();

            println!("Peer ID: {}", hex::encode(handshake_response.peer_id));
        }

        Command::DownloadPiece {
            output,
            torrent,
            piece_index,
        } => {
            let contents = fs::read(&torrent).expect("Could not read the torrent file");
            let torrent = TorrentFile::from_u8_vec(contents);

            let data = download_piece(&torrent, piece_index).await?;

            fs::write(&output, data).expect("failed to write the data to file");
            println!(
                "Piece {} downloaded to {}.",
                piece_index,
                &output.to_str().unwrap()
            );
        }

        Command::Download { output, torrent } => {
            let contents = fs::read(&torrent).expect("Could not read the torrent file");
            let torrent_file = TorrentFile::from_u8_vec(contents);
            let mut data: Vec<Vec<u8>> = vec![];
            for piece_index in 0..torrent_file.get_no_of_pieces() {
                let piece_data = download_piece(&torrent_file, piece_index).await?;
                data.push(piece_data);
            }

            fs::write(&output, data.concat()).expect("failed to write the data to file");
            println!("Downloaded {} to {}.", &torrent, &output.to_str().unwrap());
        }
    }

    Ok(())
}
