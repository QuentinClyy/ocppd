use openssl::pkey::{PKey, Private};
use openssl::x509::X509;

pub struct Identity {
    pub key: PKey<Private>,
    pub chain: Vec<X509>,
}

pub struct ClientCertificate {
    pub chain_pem: Vec<u8>,
    pub key_pkcs8_pem: Vec<u8>,
}

impl Identity {
    pub fn client_certificate(&self) -> Option<ClientCertificate> {
        let mut chain_pem = Vec::new();
        for certificate in &self.chain {
            chain_pem.extend(certificate.to_pem().ok()?);
        }
        let key_pkcs8_pem = self.key.private_key_to_pem_pkcs8().ok()?;
        Some(ClientCertificate {
            chain_pem,
            key_pkcs8_pem,
        })
    }
}
