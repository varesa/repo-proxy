use rcgen::{CertificateParams, IsCa, DistinguishedName, Certificate, RcgenError};
use crate::config::Config;

mod config;

fn build_ca() -> Result<Certificate, RcgenError> {
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "repo-proxy");

    let mut params = CertificateParams::new(Vec::new());
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    Certificate::from_params(params)
}

fn main() -> Result<(), anyhow::Error> {
    let config = Config::try_from_args()?;
    Ok(())
}
