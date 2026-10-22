use super::security_event::INVALID_CHARGE_POINT_CERTIFICATE;
use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::configuration::CPO_NAME;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::security::certificates::{
    CertificateSignedRequest, CertificateSignedResponse, SignCertificateRequest,
    SignCertificateResponse,
};
use crate::wire::ci_string::CiString;
use crate::x509::signing::{chain_fits, signing_request};
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
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
        let identity = key.and_then(|key| Some(key).filter(|key| chain_fits(key, &chain, now)));
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

#[cfg(test)]
#[path = "charge_point_certificate.test.rs"]
mod tests;
