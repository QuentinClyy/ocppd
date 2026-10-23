use super::certificate::{signed_by, valid_at};
use openssl::ec::{EcGroup, EcKey};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::{X509, X509NameBuilder, X509ReqBuilder};

pub fn signing_request(
    common_name: &str,
    organization: &str,
) -> Result<(PKey<Private>, String), ErrorStack> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let key = PKey::from_ec_key(EcKey::generate(&group)?)?;
    let mut name = X509NameBuilder::new()?;
    if !organization.is_empty() {
        name.append_entry_by_nid(Nid::ORGANIZATIONNAME, organization)?;
    }
    name.append_entry_by_nid(Nid::COMMONNAME, common_name)?;
    let mut request = X509ReqBuilder::new()?;
    request.set_subject_name(&name.build())?;
    request.set_pubkey(&key)?;
    request.sign(&key, MessageDigest::sha256())?;
    let pem = String::from_utf8_lossy(&request.build().to_pem()?).into_owned();
    Ok((key, pem))
}

pub fn chain_fits(key: &PKey<Private>, chain: &[X509], now: u64) -> bool {
    let Some(leaf) = chain.first() else {
        return false;
    };
    let same_key = leaf.public_key().is_ok_and(|public| public.public_eq(key));
    let linked = chain.windows(2).all(|pair| signed_by(&pair[0], &pair[1]));
    same_key && linked && valid_at(leaf, now)
}
