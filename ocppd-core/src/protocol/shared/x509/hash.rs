use openssl::hash::{MessageDigest, hash};
use openssl::x509::X509Ref;

const SEQUENCE: u8 = 0x30;
const BIT_STRING: u8 = 0x03;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Digest {
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashData {
    pub issuer_name_hash: String,
    pub issuer_key_hash: String,
    pub serial_number: String,
}

pub fn hash_data(root: &X509Ref, digest: Digest) -> Option<HashData> {
    let digest = match digest {
        Digest::Sha256 => MessageDigest::sha256(),
        Digest::Sha384 => MessageDigest::sha384(),
        Digest::Sha512 => MessageDigest::sha512(),
    };
    let name = root.issuer_name().to_der().ok()?;
    let key_info = root.public_key().ok()?.public_key_to_der().ok()?;
    let key = public_key_bits(&key_info)?;
    let serial = root.serial_number().to_bn().ok()?.to_hex_str().ok()?;
    Some(HashData {
        issuer_name_hash: hex(&hash(digest, &name).ok()?),
        issuer_key_hash: hex(&hash(digest, key).ok()?),
        serial_number: without_leading_zeroes(&serial),
    })
}

fn public_key_bits(subject_public_key_info: &[u8]) -> Option<&[u8]> {
    let (SEQUENCE, fields, _) = element(subject_public_key_info)? else {
        return None;
    };
    let (SEQUENCE, _algorithm, rest) = element(fields)? else {
        return None;
    };
    let (BIT_STRING, bit_string, _) = element(rest)? else {
        return None;
    };
    let (_unused_bits, key) = bit_string.split_first()?;
    Some(key)
}

fn element(der: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = der.split_first()?;
    let (&first, rest) = rest.split_first()?;
    let (length, rest) = match first {
        short if short < 0x80 => (usize::from(short), rest),
        long => {
            let count = usize::from(long & 0x7f);
            let bytes = rest.get(..count)?;
            let length = bytes
                .iter()
                .fold(0usize, |length, &byte| length << 8 | usize::from(byte));
            (length, &rest[count..])
        }
    };
    Some((tag, rest.get(..length)?, &rest[length..]))
}

fn without_leading_zeroes(serial: &str) -> String {
    match serial.trim_start_matches('0') {
        "" => "0".into(),
        digits => digits.to_lowercase(),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
