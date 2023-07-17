use rcgen::{CertificateParams, IsCa, DistinguishedName, Certificate, RcgenError};

fn build_ca() -> Result<Certificate, RcgenError> {
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "repo-proxy");

    let mut params = CertificateParams::new(Vec::new());
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    Certificate::from_params(params)
}

fn main() {
    println!("Hello, world!");
}
