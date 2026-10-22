use openssl::asn1::Asn1Time;
use openssl::x509::{X509, X509Ref};
use std::cmp::Ordering;

pub fn self_signed_root(pem: &str, now: u64) -> Option<X509> {
    let certificate = X509::from_pem(pem.as_bytes()).ok()?;
    Some(certificate)
        .filter(|certificate| signed_by(certificate, certificate) && valid_at(certificate, now))
}

pub fn signed_by(certificate: &X509Ref, issuer: &X509Ref) -> bool {
    let key = issuer.public_key();
    key.is_ok_and(|key| certificate.verify(&key).unwrap_or(false))
}

pub fn valid_at(certificate: &X509Ref, now: u64) -> bool {
    let Ok(now) = Asn1Time::from_unix((now / 1000) as i64) else {
        return false;
    };
    let started = certificate.not_before().compare(&now);
    let unexpired = certificate.not_after().compare(&now);
    started.is_ok_and(|order| order != Ordering::Greater)
        && unexpired.is_ok_and(|order| order != Ordering::Less)
}
