use super::ChargingStation;
use super::security::{SECURITY_CTRLR, SECURITY_PROFILE};
use crate::api::io::Output;
use crate::v201::event::EventKind;
use crate::v201::messages::certificates::{
    CertificateHashData, CertificateHashDataChain, DeleteCertificateRequest,
    DeleteCertificateResponse, DeleteCertificateStatus, GetCertificateIdUse,
    GetInstalledCertificateIdsRequest, GetInstalledCertificateIdsResponse,
    GetInstalledCertificateStatus, HashAlgorithm, InstallCertificateRequest,
    InstallCertificateResponse, InstallCertificateStatus, InstallCertificateUse,
};
use crate::wire::ci_string::CiString;
use crate::x509::certificate::{signed_by, valid_at};
use crate::x509::hash::{Digest, hash_data};
use openssl::x509::{X509, X509Ref};

pub const CAPACITY: usize = 10;
pub const ADDITIONAL_ROOT_CERTIFICATE_CHECK: &str = "AdditionalRootCertificateCheck";

#[derive(Default)]
pub struct Certificates {
    installed: Vec<Installed>,
}

struct Installed {
    kind: InstallCertificateUse,
    certificate: X509,
    fallback: bool,
}

impl ChargingStation {
    pub(super) fn install_certificate(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: InstallCertificateRequest,
    ) -> InstallCertificateResponse {
        let kind = request.certificate_type;
        let status = self.install(kind, request.certificate.as_str(), now);
        if status == InstallCertificateStatus::Accepted {
            self.emit(
                out,
                now,
                EventKind::CertificateInstalled {
                    certificate_type: kind,
                },
            );
        }
        InstallCertificateResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn installed_certificate_ids(
        &self,
        request: GetInstalledCertificateIdsRequest,
    ) -> GetInstalledCertificateIdsResponse {
        let wanted = request.certificate_type.unwrap_or_default();
        let chain: Vec<CertificateHashDataChain> = self
            .certificates
            .installed
            .iter()
            .filter_map(|installed| {
                let certificate_type = id_use(installed.kind);
                let listed = wanted.is_empty() || wanted.contains(&certificate_type);
                let certificate_hash_data =
                    listed.then(|| hash(&installed.certificate, HashAlgorithm::Sha256))??;
                Some(CertificateHashDataChain {
                    certificate_hash_data,
                    certificate_type,
                    child_certificate_hash_data: None,
                })
            })
            .collect();
        let status = match chain.is_empty() {
            true => GetInstalledCertificateStatus::NotFound,
            false => GetInstalledCertificateStatus::Accepted,
        };
        GetInstalledCertificateIdsResponse {
            status,
            status_info: None,
            certificate_hash_data_chain: (!chain.is_empty()).then_some(chain),
        }
    }

    pub(super) fn delete_certificate(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: DeleteCertificateRequest,
    ) -> DeleteCertificateResponse {
        let wanted = request.certificate_hash_data;
        let found = self.certificates.installed.iter().position(|installed| {
            hash(&installed.certificate, wanted.hash_algorithm)
                .is_some_and(|data| same(&data, &wanted))
        });
        let status = match found {
            None => DeleteCertificateStatus::NotFound,
            Some(index) if self.last_csms_root_in_use(index) => DeleteCertificateStatus::Failed,
            Some(index) => {
                let removed = self.certificates.installed.remove(index);
                let certificate_type = removed.kind;
                self.emit(out, now, EventKind::CertificateDeleted { certificate_type });
                DeleteCertificateStatus::Accepted
            }
        };
        DeleteCertificateResponse {
            status,
            status_info: None,
        }
    }

    pub fn csms_roots(&self) -> Vec<Vec<u8>> {
        self.certificates
            .installed
            .iter()
            .filter(|installed| installed.kind == InstallCertificateUse::CsmsRootCertificate)
            .filter_map(|installed| installed.certificate.to_pem().ok())
            .collect()
    }

    pub(super) fn drop_fallback_roots(&mut self) {
        self.certificates
            .installed
            .retain(|installed| !installed.fallback);
    }

    pub(super) fn trusted_firmware_signer(&self, pem: &str, now: u64) -> bool {
        let Ok(signer) = X509::from_pem(pem.as_bytes()) else {
            return false;
        };
        valid_at(&signer, now)
            && self.certificates.installed.iter().any(|installed| {
                installed.kind == InstallCertificateUse::ManufacturerRootCertificate
                    && signed_by(&signer, &installed.certificate)
            })
    }

    pub(super) fn certificate_count(&self) -> usize {
        self.certificates.installed.len()
    }

    fn install(
        &mut self,
        kind: InstallCertificateUse,
        pem: &str,
        now: u64,
    ) -> InstallCertificateStatus {
        let parsed = X509::from_pem(pem.as_bytes()).ok();
        let Some(certificate) = parsed.filter(|certificate| valid_at(certificate, now)) else {
            return InstallCertificateStatus::Rejected;
        };
        let replaced = self.replaced_root(kind);
        let issuer = replaced.map(|index| &self.certificates.installed[index].certificate);
        if !signed_by(&certificate, issuer.unwrap_or(&certificate)) {
            return InstallCertificateStatus::Rejected;
        }
        let installed = &mut self.certificates.installed;
        if replaced.is_some() {
            installed.retain(|other| !other.fallback);
            if let Some(old) = installed
                .iter_mut()
                .find(|other| other.kind == InstallCertificateUse::CsmsRootCertificate)
            {
                old.fallback = true;
            }
        }
        let duplicate = installed.iter().position(|other| {
            other.kind == kind && other.certificate.to_der().ok() == certificate.to_der().ok()
        });
        if let Some(index) = duplicate {
            installed.remove(index);
        } else if installed.len() >= CAPACITY {
            return InstallCertificateStatus::Rejected;
        }
        installed.push(Installed {
            kind,
            certificate,
            fallback: false,
        });
        InstallCertificateStatus::Accepted
    }

    fn replaced_root(&self, kind: InstallCertificateUse) -> Option<usize> {
        let checked = self
            .device_model
            .value(SECURITY_CTRLR, ADDITIONAL_ROOT_CERTIFICATE_CHECK)
            == Some("true");
        let root = kind == InstallCertificateUse::CsmsRootCertificate;
        let current = self.certificates.installed.iter().position(|other| {
            other.kind == InstallCertificateUse::CsmsRootCertificate && !other.fallback
        });
        current.filter(|_| checked && root)
    }

    fn last_csms_root_in_use(&self, index: usize) -> bool {
        let installed = &self.certificates.installed;
        let csms_roots = installed
            .iter()
            .filter(|other| other.kind == InstallCertificateUse::CsmsRootCertificate)
            .count();
        let profile = self
            .device_model
            .number(SECURITY_CTRLR, SECURITY_PROFILE)
            .unwrap_or(0);
        installed[index].kind == InstallCertificateUse::CsmsRootCertificate
            && csms_roots == 1
            && profile >= 2
    }
}

fn id_use(kind: InstallCertificateUse) -> GetCertificateIdUse {
    match kind {
        InstallCertificateUse::V2gRootCertificate => GetCertificateIdUse::V2gRootCertificate,
        InstallCertificateUse::MoRootCertificate => GetCertificateIdUse::MoRootCertificate,
        InstallCertificateUse::CsmsRootCertificate => GetCertificateIdUse::CsmsRootCertificate,
        InstallCertificateUse::ManufacturerRootCertificate => {
            GetCertificateIdUse::ManufacturerRootCertificate
        }
    }
}

fn hash(root: &X509Ref, algorithm: HashAlgorithm) -> Option<CertificateHashData> {
    let digest = match algorithm {
        HashAlgorithm::Sha256 => Digest::Sha256,
        HashAlgorithm::Sha384 => Digest::Sha384,
        HashAlgorithm::Sha512 => Digest::Sha512,
    };
    let data = hash_data(root, digest)?;
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
