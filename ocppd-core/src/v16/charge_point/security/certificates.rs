use super::hash_data::{hash_data, same};
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::charge_point::ChargePoint;
use crate::v16::charge_point::configuration::CERTIFICATE_STORE_MAX_LENGTH;
use crate::v16::messages::security::certificates::{
    CertificateHashData, CertificateUse, DeleteCertificateRequest, DeleteCertificateResponse,
    DeleteCertificateStatus, GetInstalledCertificateIdsRequest, GetInstalledCertificateIdsResponse,
    GetInstalledCertificateStatus, HashAlgorithm, InstallCertificateRequest,
    InstallCertificateResponse, InstallCertificateStatus,
};
use openssl::asn1::Asn1Time;
use openssl::x509::X509;
use std::cmp::Ordering;

#[derive(Default)]
pub struct Certificates {
    installed: Vec<Installed>,
}

struct Installed {
    certificate_type: CertificateUse,
    certificate: X509,
    hash_data: CertificateHashData,
}

impl ChargePoint {
    pub(crate) fn install_certificate(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: InstallCertificateRequest,
    ) -> InstallCertificateResponse {
        let certificate_type = request.certificate_type;
        let max = self
            .configuration
            .number(CERTIFICATE_STORE_MAX_LENGTH)
            .unwrap_or(0) as usize;
        let installed = root(request.certificate.as_str(), now).and_then(|certificate| {
            self.certificates
                .install(certificate_type, certificate, max)
        });
        let status = match installed {
            Some(()) => InstallCertificateStatus::Accepted,
            None => InstallCertificateStatus::Rejected,
        };
        if status == InstallCertificateStatus::Accepted {
            let kind = EventKind::CertificateInstalled { certificate_type };
            self.emit(out, now, kind);
        }
        InstallCertificateResponse { status }
    }

    pub(crate) fn installed_certificate_ids(
        &self,
        request: GetInstalledCertificateIdsRequest,
    ) -> GetInstalledCertificateIdsResponse {
        let ids: Vec<CertificateHashData> = self
            .certificates
            .installed
            .iter()
            .filter(|installed| installed.certificate_type == request.certificate_type)
            .map(|installed| installed.hash_data.clone())
            .collect();
        match ids.is_empty() {
            true => GetInstalledCertificateIdsResponse {
                certificate_hash_data: None,
                status: GetInstalledCertificateStatus::NotFound,
            },
            false => GetInstalledCertificateIdsResponse {
                certificate_hash_data: Some(ids),
                status: GetInstalledCertificateStatus::Accepted,
            },
        }
    }

    pub(crate) fn delete_certificate(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: DeleteCertificateRequest,
    ) -> DeleteCertificateResponse {
        let wanted = request.certificate_hash_data;
        let found = self.certificates.installed.iter().position(|installed| {
            hash_data(&installed.certificate, wanted.hash_algorithm)
                .is_some_and(|hash_data| same(&hash_data, &wanted))
        });
        let Some(index) = found else {
            return DeleteCertificateResponse {
                status: DeleteCertificateStatus::NotFound,
            };
        };
        let removed = self.certificates.installed.remove(index);
        let certificate_type = removed.certificate_type;
        self.emit(out, now, EventKind::CertificateDeleted { certificate_type });
        DeleteCertificateResponse {
            status: DeleteCertificateStatus::Accepted,
        }
    }
}

impl Certificates {
    fn install(
        &mut self,
        certificate_type: CertificateUse,
        certificate: X509,
        max: usize,
    ) -> Option<()> {
        let hash_data = hash_data(&certificate, HashAlgorithm::Sha256)?;
        let installed = Installed {
            certificate_type,
            certificate,
            hash_data,
        };
        let existing = self
            .installed
            .iter()
            .position(|other| same(&other.hash_data, &installed.hash_data));
        match existing {
            Some(index) => self.installed[index] = installed,
            None if self.installed.len() < max => self.installed.push(installed),
            None => return None,
        }
        Some(())
    }
}

fn root(pem: &str, now: u64) -> Option<X509> {
    let certificate = X509::from_pem(pem.as_bytes()).ok()?;
    let key = certificate.public_key().ok()?;
    let self_signed = certificate.verify(&key).ok()?;
    let now = Asn1Time::from_unix((now / 1000) as i64).ok()?;
    let started = certificate.not_before().compare(&now).ok()? != Ordering::Greater;
    let unexpired = certificate.not_after().compare(&now).ok()? != Ordering::Less;
    Some(certificate).filter(|_| self_signed && started && unexpired)
}

#[cfg(test)]
#[path = "certificates.test.rs"]
mod tests;
