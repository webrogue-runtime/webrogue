use std::{fs::File, io::Write as _, path::PathBuf};

use clap::Args;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};

#[derive(Args, Debug, Clone)]
pub struct AndroidKeygen {
    pub output: PathBuf,
    pub alias: String,
    pub password: String,
}

impl AndroidKeygen {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut params = CertificateParams::default();

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Android Release Key");
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;

        let cert = params.self_signed(&key_pair)?;

        let cert_der = cert.der().to_vec();
        let pkey_der = key_pair.serialize_der();

        let p12_der = p12::PFX::new(&cert_der, &pkey_der, None, &self.password, &self.alias)
            .ok_or_else(|| anyhow::anyhow!("Failed to generate PKCS12 DER"))?
            .to_der();

        let mut file = File::create(&self.output)?;
        file.write_all(&p12_der)?;
        Ok(())
    }
}
