use super::certificates::valid_at;
use super::security_event::INVALID_CHARGE_POINT_CERTIFICATE;
use crate::api::io::Output;
use crate::v16::charge_point::ChargePoint;
use crate::v16::charge_point::configuration::CPO_NAME;
use crate::v16::event::EventKind;
use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::v16::messages::security::certificates::{
    CertificateSignedRequest, CertificateSignedResponse, SignCertificateRequest,
    SignCertificateResponse,
};
use crate::wire::ci_string::CiString;
use openssl::ec::{EcGroup, EcKey};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::{X509, X509NameBuilder, X509ReqBuilder};
use serde_json::{Value, json};

#[derive(Default)]
pub struct ChargePointCertificate {
    pending_key: Option<PKey<Private>>,
    installed: Option<Identity>,
}

struct Identity {
    key: PKey<Private>,
    chain: Vec<X509>,
}

pub struct ClientCertificate {
    pub chain_pem: Vec<u8>,
    pub key_pkcs8_pem: Vec<u8>,
}

impl ChargePoint {
    pub fn client_certificate(&self) -> Option<ClientCertificate> {
        if !self.requires_client_certificate() {
            return None;
        }
        let identity = self.client_certificate.installed.as_ref()?;
        let mut chain_pem = Vec::new();
        for certificate in &identity.chain {
            chain_pem.extend(certificate.to_pem().ok()?);
        }
        let key_pkcs8_pem = identity.key.private_key_to_pem_pkcs8().ok()?;
        Some(ClientCertificate {
            chain_pem,
            key_pkcs8_pem,
        })
    }

    pub(crate) fn has_charge_point_certificate(&self) -> bool {
        self.client_certificate.installed.is_some()
    }

    pub(crate) fn sign_charge_point_certificate(&mut self) -> bool {
        let cpo_name = self.configuration.text(CPO_NAME).unwrap_or_default();
        let Ok((key, csr)) = signing_request(&self.station.serial_number, cpo_name) else {
            return false;
        };
        let Ok(csr) = CiString::new(&csr) else {
            return false;
        };
        self.client_certificate.pending_key = Some(key);
        self.outbox
            .push("SignCertificate", 0, json!(SignCertificateRequest { csr }));
        true
    }

    pub(crate) fn sign_certificate_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        payload: Value,
    ) {
        let response = self.decode::<SignCertificateResponse>(out, now, "SignCertificate", payload);
        if response.is_none_or(|response| response.status == GenericStatus::Rejected) {
            self.client_certificate.pending_key = None;
        }
    }

    pub(crate) fn certificate_signed(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CertificateSignedRequest,
    ) -> CertificateSignedResponse {
        let pem = request.certificate_chain.as_str().as_bytes();
        let chain = X509::stack_from_pem(pem).unwrap_or_default();
        let key = self.client_certificate.pending_key.take();
        let identity = key.and_then(|key| Some(key).filter(|key| fits(key, &chain, now)));
        let Some(key) = identity else {
            self.security_event(out, now, INVALID_CHARGE_POINT_CERTIFICATE);
            return CertificateSignedResponse {
                status: GenericStatus::Rejected,
            };
        };
        self.client_certificate.installed = Some(Identity { key, chain });
        self.emit(out, now, EventKind::ChargePointCertificateInstalled);
        CertificateSignedResponse {
            status: GenericStatus::Accepted,
        }
    }
}

fn signing_request(
    serial_number: &str,
    cpo_name: &str,
) -> Result<(PKey<Private>, String), ErrorStack> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let key = PKey::from_ec_key(EcKey::generate(&group)?)?;
    let mut name = X509NameBuilder::new()?;
    if !cpo_name.is_empty() {
        name.append_entry_by_nid(Nid::ORGANIZATIONNAME, cpo_name)?;
    }
    name.append_entry_by_nid(Nid::COMMONNAME, serial_number)?;
    let mut request = X509ReqBuilder::new()?;
    request.set_subject_name(&name.build())?;
    request.set_pubkey(&key)?;
    request.sign(&key, MessageDigest::sha256())?;
    let pem = String::from_utf8_lossy(&request.build().to_pem()?).into_owned();
    Ok((key, pem))
}

fn fits(key: &PKey<Private>, chain: &[X509], now: u64) -> bool {
    let Some(leaf) = chain.first() else {
        return false;
    };
    let same_key = leaf.public_key().is_ok_and(|public| public.public_eq(key));
    let linked = chain.windows(2).all(|pair| {
        let issuer = pair[1].public_key();
        issuer.is_ok_and(|issuer| pair[0].verify(&issuer).unwrap_or(false))
    });
    same_key && linked && valid_at(leaf, now)
}

#[cfg(test)]
#[path = "charge_point_certificate.test.rs"]
mod tests;
