use crate::v16::messages::security::certificates::{CertificateHashData, HashAlgorithm};
use crate::wire::ci_string::CiString;
use openssl::hash::{MessageDigest, hash};
use openssl::x509::X509Ref;

const SEQUENCE: u8 = 0x30;
const BIT_STRING: u8 = 0x03;

pub fn hash_data(root: &X509Ref, algorithm: HashAlgorithm) -> Option<CertificateHashData> {
    let digest = match algorithm {
        HashAlgorithm::Sha256 => MessageDigest::sha256(),
        HashAlgorithm::Sha384 => MessageDigest::sha384(),
        HashAlgorithm::Sha512 => MessageDigest::sha512(),
    };
    let name = root.issuer_name().to_der().ok()?;
    let key_info = root.public_key().ok()?.public_key_to_der().ok()?;
    let key = public_key_bits(&key_info)?;
    let serial = root.serial_number().to_bn().ok()?.to_hex_str().ok()?;
    Some(CertificateHashData {
        hash_algorithm: algorithm,
        issuer_name_hash: CiString::new(&hex(&hash(digest, &name).ok()?)).ok()?,
        issuer_key_hash: CiString::new(&hex(&hash(digest, key).ok()?)).ok()?,
        serial_number: CiString::new(&without_leading_zeroes(&serial)).ok()?,
    })
}

pub fn same(left: &CertificateHashData, right: &CertificateHashData) -> bool {
    left.hash_algorithm == right.hash_algorithm
        && left.issuer_name_hash.matches(&right.issuer_name_hash)
        && left.issuer_key_hash.matches(&right.issuer_key_hash)
        && left.serial_number.matches(&right.serial_number)
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
