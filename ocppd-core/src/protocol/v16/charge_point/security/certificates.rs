use crate::api::io::Output;
use crate::protocol::shared::x509::certificate::self_signed_root;
use crate::protocol::shared::x509::hash::{self, Digest};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::configuration::CERTIFICATE_STORE_MAX_LENGTH;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::security::certificates::{
    CertificateHashData, CertificateUse, DeleteCertificateRequest, DeleteCertificateResponse,
    DeleteCertificateStatus, GetInstalledCertificateIdsRequest, GetInstalledCertificateIdsResponse,
    GetInstalledCertificateStatus, HashAlgorithm, InstallCertificateRequest,
    InstallCertificateResponse, InstallCertificateStatus,
};
use crate::wire::ci_string::CiString;
use openssl::x509::{X509, X509Ref};

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
        let installed =
            self_signed_root(request.certificate.as_str(), now).and_then(|certificate| {
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

    pub fn csms_roots(&self) -> Vec<Vec<u8>> {
        let roots = self.certificates.installed.iter().filter(|installed| {
            installed.certificate_type == CertificateUse::CentralSystemRootCertificate
        });
        roots
            .filter_map(|installed| installed.certificate.to_pem().ok())
            .collect()
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

fn hash_data(root: &X509Ref, algorithm: HashAlgorithm) -> Option<CertificateHashData> {
    let digest = match algorithm {
        HashAlgorithm::Sha256 => Digest::Sha256,
        HashAlgorithm::Sha384 => Digest::Sha384,
        HashAlgorithm::Sha512 => Digest::Sha512,
    };
    let data = hash::hash_data(root, digest)?;
    Some(CertificateHashData {
        hash_algorithm: algorithm,
        issuer_name_hash: CiString::new(&data.issuer_name_hash).ok()?,
        issuer_key_hash: CiString::new(&data.issuer_key_hash).ok()?,
        serial_number: CiString::new(&data.serial_number).ok()?,
    })
}

fn same(left: &CertificateHashData, right: &CertificateHashData) -> bool {
    left.hash_algorithm == right.hash_algorithm
        && left.issuer_name_hash.matches(&right.issuer_name_hash)
        && left.issuer_key_hash.matches(&right.issuer_key_hash)
        && left.serial_number.matches(&right.serial_number)
}

#[cfg(test)]
#[path = "certificates.test.rs"]
mod tests;
