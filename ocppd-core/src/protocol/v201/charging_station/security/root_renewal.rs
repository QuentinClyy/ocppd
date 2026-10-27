use super::certificates::ADDITIONAL_ROOT_CERTIFICATE_CHECK;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::security::SECURITY_CTRLR;
use crate::protocol::v201::messages::security::certificates::InstallCertificateUse;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn csms_root_refused(&mut self) {
        let certificates = &mut self.certificates;
        let has_fallback = certificates
            .installed
            .iter()
            .any(|installed| installed.fallback);
        certificates.on_fallback = has_fallback && !certificates.on_fallback;
    }

    pub(in crate::protocol::v201::charging_station) fn settle_csms_roots(&mut self) {
        let certificates = &mut self.certificates;
        let kept = certificates.on_fallback;
        certificates.installed.retain(|installed| {
            installed.kind != InstallCertificateUse::CsmsRootCertificate
                || installed.fallback == kept
        });
        for installed in &mut certificates.installed {
            installed.fallback = false;
        }
        certificates.on_fallback = false;
    }

    pub(super) fn replaced_root(&self, kind: InstallCertificateUse) -> Option<usize> {
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
}

#[cfg(test)]
#[path = "root_renewal.test.rs"]
mod tests;
