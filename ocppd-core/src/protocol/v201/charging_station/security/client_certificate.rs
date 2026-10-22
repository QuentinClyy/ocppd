use super::{
    CERT_SIGNING_REPEAT_TIMES, CERT_SIGNING_WAIT_MINIMUM, INVALID_CHARGING_STATION_CERTIFICATE,
    ORGANIZATION_NAME, SECURITY_CTRLR,
};
use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v201::messages::security::certificates::{
    CertificateSignedRequest, CertificateSignedResponse, CertificateSignedStatus,
    CertificateSigningUse, SignCertificateRequest, SignCertificateResponse,
};
use crate::wire::ci_string::CiString;
use crate::x509::certificate::{expires_at, subject_entry};
use crate::x509::identity::{ClientCertificate, Identity};
use crate::x509::signing::{chain_fits, signing_request};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use serde_json::{Value, json};

const RENEW_BEFORE_MS: u64 = 30 * 86_400_000;

#[derive(Default)]
pub struct ClientCertificates {
    pending: Option<Pending>,
    installed: Option<Identity>,
    renewal_requested: bool,
    reconnect: bool,
}

struct Pending {
    key: PKey<Private>,
    csr: CiString<5500>,
    wait_s: u64,
    repeats_left: u64,
    resend_at: Option<u64>,
}

impl ChargingStation {
    pub fn client_certificate(&self) -> Option<ClientCertificate> {
        if !self.requires_client_certificate() {
            return None;
        }
        self.client_certificate
            .installed
            .as_ref()?
            .client_certificate()
    }

    pub(in crate::protocol::v201::charging_station) fn has_client_certificate(&self) -> bool {
        self.client_certificate.installed.is_some()
    }

    pub(in crate::protocol::v201::charging_station) fn sign_charging_station_certificate(
        &mut self,
    ) -> bool {
        let organization = self
            .device_model
            .value(SECURITY_CTRLR, ORGANIZATION_NAME)
            .unwrap_or_default();
        let Ok((key, csr)) = signing_request(&self.station.serial_number, organization) else {
            return false;
        };
        let Ok(csr) = CiString::new(&csr) else {
            return false;
        };
        let setting = |name| self.device_model.number(SECURITY_CTRLR, name).unwrap_or(0);
        self.client_certificate.pending = Some(Pending {
            key,
            csr: csr.clone(),
            wait_s: setting(CERT_SIGNING_WAIT_MINIMUM),
            repeats_left: setting(CERT_SIGNING_REPEAT_TIMES) + 1,
            resend_at: None,
        });
        self.send_sign_certificate(csr);
        true
    }

    pub(in crate::protocol::v201::charging_station) fn sign_certificate_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        payload: Value,
    ) {
        let response = self.decode::<SignCertificateResponse>(out, now, "SignCertificate", payload);
        if response.is_none_or(|response| response.status == GenericStatus::Rejected) {
            self.client_certificate.pending = None;
            return;
        }
        if let Some(pending) = &mut self.client_certificate.pending {
            pending.resend_at = Some(now + pending.wait_s * 1000);
        }
    }

    pub(in crate::protocol::v201::charging_station) fn sign_certificate_failed(&mut self) {
        self.client_certificate.pending = None;
    }

    pub(in crate::protocol::v201::charging_station) fn certificate_signed(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CertificateSignedRequest,
    ) -> CertificateSignedResponse {
        let pem = request.certificate_chain.as_str().as_bytes();
        let chain = X509::stack_from_pem(pem).unwrap_or_default();
        let station_certificate = request
            .certificate_type
            .is_none_or(|kind| kind == CertificateSigningUse::ChargingStationCertificate);
        let pending = self.client_certificate.pending.take();
        let key = pending
            .map(|pending| pending.key)
            .filter(|key| station_certificate && chain_fits(key, &chain, now))
            .filter(|_| self.owns(&chain));
        let Some(key) = key else {
            self.security_event(out, now, INVALID_CHARGING_STATION_CERTIFICATE);
            return response(CertificateSignedStatus::Rejected);
        };
        let certificates = &mut self.client_certificate;
        certificates.installed = Some(Identity { key, chain });
        certificates.renewal_requested = false;
        certificates.reconnect = true;
        self.emit(out, now, EventKind::ChargingStationCertificateInstalled);
        response(CertificateSignedStatus::Accepted)
    }

    pub(in crate::protocol::v201::charging_station) fn client_certificate_deadline(
        &self,
    ) -> Option<u64> {
        let certificates = &self.client_certificate;
        let resend = certificates
            .pending
            .as_ref()
            .and_then(|pending| pending.resend_at);
        let renewal = self
            .renewal_at()
            .filter(|_| !certificates.renewal_requested);
        resend.into_iter().chain(renewal).min()
    }

    pub(in crate::protocol::v201::charging_station) fn client_certificate_tick(
        &mut self,
        now: u64,
    ) {
        let renewal_due = self.renewal_at().is_some_and(|at| now >= at);
        let certificates = &mut self.client_certificate;
        if renewal_due && !certificates.renewal_requested && certificates.pending.is_none() {
            certificates.renewal_requested = true;
            self.sign_charging_station_certificate();
            return;
        }
        let Some(pending) = &mut self.client_certificate.pending else {
            return;
        };
        if pending.resend_at.is_none_or(|at| now < at) {
            return;
        }
        pending.resend_at = None;
        if pending.repeats_left == 0 {
            return;
        }
        pending.repeats_left -= 1;
        pending.wait_s *= 2;
        let csr = pending.csr.clone();
        self.send_sign_certificate(csr);
    }

    pub(in crate::protocol::v201::charging_station) fn reconnect_due(&mut self, now: u64) -> bool {
        let due = self.client_certificate.reconnect && self.outbox.is_drained(now);
        if due {
            self.client_certificate.reconnect = false;
        }
        due
    }

    fn owns(&self, chain: &[X509]) -> bool {
        let Some(leaf) = chain.first() else {
            return false;
        };
        let organization = self
            .device_model
            .value(SECURITY_CTRLR, ORGANIZATION_NAME)
            .unwrap_or_default();
        let common_name = subject_entry(leaf, Nid::COMMONNAME);
        let organization_name = subject_entry(leaf, Nid::ORGANIZATIONNAME).unwrap_or_default();
        common_name.as_deref() == Some(self.station.serial_number.as_str())
            && organization_name == organization
    }

    fn renewal_at(&self) -> Option<u64> {
        let installed = self.client_certificate.installed.as_ref()?;
        let leaf = installed.chain.first()?;
        Some(expires_at(leaf)?.saturating_sub(RENEW_BEFORE_MS))
    }

    fn send_sign_certificate(&mut self, csr: CiString<5500>) {
        let request = SignCertificateRequest {
            csr,
            certificate_type: Some(CertificateSigningUse::ChargingStationCertificate),
        };
        self.outbox.push("SignCertificate", 0, json!(request));
    }
}

fn response(status: CertificateSignedStatus) -> CertificateSignedResponse {
    CertificateSignedResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "client_certificate.test.rs"]
mod tests;
