use std::path::Path;
use rcgen::{CertificateParams, IsCa, DistinguishedName, Certificate, RcgenError};

const KEY_FILE: &str = "ca.key";
const CERT_FILE: &str = "ca.crt";

fn new_keypair() -> Result<Certificate, RcgenError> {
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "repo-proxy");

    let mut params = CertificateParams::new(Vec::new());
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    Certificate::from_params(params)
}

pub struct Ca {
    certificate: Certificate
}

impl Ca {
    pub fn get_or_create(datadir: &Path) -> Result<Self, anyhow::Error> {
        let private_key_path = datadir.join(KEY_FILE);
        let public_key_path = datadir.join(CERT_FILE);
        assert_eq!(private_key_path.is_file(), public_key_path.is_file());
        if public_key_path.is_file() {
            todo!()
        } else {
            let certificate = new_keypair()?;
            let private_key_data = certificate.serialize_private_key_pem();
            let public_key_data = certificate.serialize_pem()?;

            std::fs::write(private_key_path, private_key_data)?;
            std::fs::write(public_key_path, public_key_data)?;

            Ok(Ca {
                certificate
            })
        }
    }
}

