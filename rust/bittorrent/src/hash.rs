use sha1::{Digest, Sha1};

pub fn b_sha1(bytes: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();

    hasher.update(bytes);

    Vec::from(&hasher.finalize()[..])
}

pub fn hex_sha1(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);

    hex::encode(&hasher.finalize()[..])
}
