use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::{X509, X509NameBuilder, X509NameRef, X509Req};

const VALID_UNTIL_S: i64 = 4_000_000_000;

pub struct Authority {
    key: PKey<Private>,
    pub certificate: X509,
}

impl Authority {
    pub fn new(name: &str) -> Authority {
        let key = new_key();
        let mut subject = X509NameBuilder::new().unwrap();
        subject.append_entry_by_text("CN", name).unwrap();
        let subject = subject.build();
        let certificate = issue(&subject, &subject, &key, &key);
        Authority { key, certificate }
    }

    pub fn pem(&self) -> String {
        String::from_utf8(self.certificate.to_pem().unwrap()).unwrap()
    }

    pub fn sign(&self, csr: &str) -> X509 {
        let request = X509Req::from_pem(csr.as_bytes()).unwrap();
        let public = request.public_key().unwrap();
        let issuer = self.certificate.subject_name();
        issue(request.subject_name(), issuer, &public, &self.key)
    }
}

pub fn new_key() -> PKey<Private> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    PKey::from_ec_key(EcKey::generate(&group).unwrap()).unwrap()
}

fn issue<T: openssl::pkey::HasPublic>(
    subject: &X509NameRef,
    issuer: &X509NameRef,
    public: &openssl::pkey::PKeyRef<T>,
    signer: &PKey<Private>,
) -> X509 {
    let mut builder = X509::builder().unwrap();
    let serial = BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap();
    builder.set_serial_number(&serial).unwrap();
    builder.set_subject_name(subject).unwrap();
    builder.set_issuer_name(issuer).unwrap();
    builder.set_pubkey(public).unwrap();
    builder
        .set_not_before(&Asn1Time::from_unix(0).unwrap())
        .unwrap();
    builder
        .set_not_after(&Asn1Time::from_unix(VALID_UNTIL_S).unwrap())
        .unwrap();
    builder.sign(signer, MessageDigest::sha256()).unwrap();
    builder.build()
}
