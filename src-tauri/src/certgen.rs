use chrono::{Duration, Utc};
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    rsa::Rsa,
    sha::{sha1, sha256},
    x509::{
        extension::{
            AuthorityKeyIdentifier, BasicConstraints, ExtendedKeyUsage, KeyUsage,
            SubjectAlternativeName, SubjectKeyIdentifier,
        },
        X509Builder, X509Name, X509NameBuilder, X509NameRef, X509,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectInfo {
    pub common_name: String,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaParams {
    pub subject: SubjectInfo,
    pub key_algorithm: KeyAlgorithm,
    pub validity_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanEntry {
    pub dns_names: Vec<String>,
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslParams {
    pub subject: SubjectInfo,
    pub san: SanEntry,
    pub key_algorithm: KeyAlgorithm,
    pub validity_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub cert_pem: String,
    pub key_pem: String,
    pub subject: String,
    pub issuer: Option<String>,
    pub serial_number: String,
    pub valid_from: String,
    pub valid_to: String,
    pub sha256_fingerprint: String,
    pub sha1_fingerprint: String,
    pub key_algorithm: String,
    pub san: Option<Vec<String>>,
}

fn algorithm_name(alg: &KeyAlgorithm) -> String {
    match alg {
        KeyAlgorithm::Rsa2048 => "RSA 2048 bits".to_string(),
        KeyAlgorithm::Rsa4096 => "RSA 4096 bits".to_string(),
        KeyAlgorithm::EcdsaP256 => "ECDSA P-256".to_string(),
        KeyAlgorithm::EcdsaP384 => "ECDSA P-384".to_string(),
    }
}

fn generate_pkey(alg: &KeyAlgorithm) -> Result<PKey<Private>, String> {
    match alg {
        KeyAlgorithm::Rsa2048 => {
            let rsa =
                Rsa::generate(2048).map_err(|e| format!("RSA 2048 密钥生成失败: {}", e))?;
            PKey::from_rsa(rsa).map_err(|e| format!("PKey 创建失败: {}", e))
        }
        KeyAlgorithm::Rsa4096 => {
            let rsa =
                Rsa::generate(4096).map_err(|e| format!("RSA 4096 密钥生成失败: {}", e))?;
            PKey::from_rsa(rsa).map_err(|e| format!("PKey 创建失败: {}", e))
        }
        KeyAlgorithm::EcdsaP256 => {
            let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)
                .map_err(|e| format!("EC 组创建失败: {}", e))?;
            let ec_key =
                EcKey::generate(&group).map_err(|e| format!("EC P-256 密钥生成失败: {}", e))?;
            PKey::from_ec_key(ec_key).map_err(|e| format!("PKey 创建失败: {}", e))
        }
        KeyAlgorithm::EcdsaP384 => {
            let group =
                EcGroup::from_curve_name(Nid::SECP384R1).map_err(|e| format!("EC 组创建失败: {}", e))?;
            let ec_key =
                EcKey::generate(&group).map_err(|e| format!("EC P-384 密钥生成失败: {}", e))?;
            PKey::from_ec_key(ec_key).map_err(|e| format!("PKey 创建失败: {}", e))
        }
    }
}

fn build_subject_name(subject: &SubjectInfo) -> Result<X509Name, String> {
    let mut name_builder = X509NameBuilder::new().map_err(|e| format!("X509Name 创建失败: {}", e))?;

    if let Some(ref c) = subject.country {
        if !c.is_empty() {
            name_builder
                .append_entry_by_text("C", c)
                .map_err(|e| format!("添加 Country 失败: {}", e))?;
        }
    }
    if let Some(ref o) = subject.organization {
        if !o.is_empty() {
            name_builder
                .append_entry_by_text("O", o)
                .map_err(|e| format!("添加 Organization 失败: {}", e))?;
        }
    }
    if let Some(ref ou) = subject.organizational_unit {
        if !ou.is_empty() {
            name_builder
                .append_entry_by_text("OU", ou)
                .map_err(|e| format!("添加 OU 失败: {}", e))?;
        }
    }
    name_builder
        .append_entry_by_text("CN", &subject.common_name)
        .map_err(|e| format!("添加 CN 失败: {}", e))?;

    Ok(name_builder.build())
}

fn subject_to_string(subject: &SubjectInfo) -> String {
    let mut parts = Vec::new();
    if let Some(ref c) = subject.country {
        if !c.is_empty() {
            parts.push(format!("C = {}", c));
        }
    }
    if let Some(ref o) = subject.organization {
        if !o.is_empty() {
            parts.push(format!("O = {}", o));
        }
    }
    if let Some(ref ou) = subject.organizational_unit {
        if !ou.is_empty() {
            parts.push(format!("OU = {}", ou));
        }
    }
    parts.push(format!("CN = {}", subject.common_name));
    parts.join(", ")
}

fn x509_name_to_string(name: &X509NameRef) -> String {
    let mut parts = Vec::new();
    for entry in name.entries() {
        if let Ok(data) = entry.data().as_utf8() {
            let obj = entry.object();
            let nid = obj.nid();
            if let Ok(short_name) = nid.short_name() {
                parts.push(format!("{} = {}", short_name, data));
            }
        }
    }
    parts.join(", ")
}

fn compute_sha256_fingerprint(der_bytes: &[u8]) -> String {
    let hash = sha256(der_bytes);
    hex::encode(hash)
        .to_uppercase()
        .as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<&str>>()
        .join(":")
}

fn compute_sha1_fingerprint(der_bytes: &[u8]) -> String {
    let hash = sha1(der_bytes);
    hex::encode(hash)
        .to_uppercase()
        .as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<&str>>()
        .join(":")
}

fn format_utc_now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn format_utc_after(days: u32) -> String {
    (Utc::now() + Duration::days(days as i64))
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string()
}

fn generate_serial() -> Result<(BigNum, String), String> {
    let mut bn = BigNum::new().map_err(|e| format!("BigNum 创建失败: {}", e))?;
    bn.rand(64, MsbOption::MAYBE_ZERO, false)
        .map_err(|e| format!("序列号随机数生成失败: {}", e))?;
    let serial_hex = hex::encode(bn.to_vec()).to_uppercase();
    Ok((bn, serial_hex))
}

/// Generate a CA (self-signed root) certificate
pub fn generate_ca(params: &CaParams) -> Result<CertInfo, String> {
    let key_pair = generate_pkey(&params.key_algorithm)?;
    let subject_name = build_subject_name(&params.subject)?;

    let mut builder =
        X509Builder::new().map_err(|e| format!("X509Builder 创建失败: {}", e))?;

    builder
        .set_version(2)
        .map_err(|e| format!("设置版本失败: {}", e))?;

    let (bn, serial_hex) = generate_serial()?;
    let asn1_int = bn.to_asn1_integer().map_err(|e| format!("序列号转换失败: {}", e))?;
    builder
        .set_serial_number(&asn1_int)
        .map_err(|e| format!("设置序列号失败: {}", e))?;

    builder
        .set_subject_name(&subject_name)
        .map_err(|e| format!("设置主题失败: {}", e))?;
    builder
        .set_issuer_name(&subject_name)
        .map_err(|e| format!("设置签发者失败: {}", e))?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|e| format!("设置生效时间失败: {}", e))?;
    let not_after = Asn1Time::days_from_now(params.validity_days as u32)
        .map_err(|e| format!("设置过期时间失败: {}", e))?;
    builder
        .set_not_before(&not_before)
        .map_err(|e| format!("设置 not_before 失败: {}", e))?;
    builder
        .set_not_after(&not_after)
        .map_err(|e| format!("设置 not_after 失败: {}", e))?;

    builder
        .set_pubkey(&key_pair)
        .map_err(|e| format!("设置公钥失败: {}", e))?;

    // Basic Constraints: CA:TRUE, pathlen:0
    let bc = BasicConstraints::new()
        .ca()
        .pathlen(0)
        .build()
        .map_err(|e| format!("BasicConstraints 构建失败: {}", e))?;
    builder
        .append_extension(bc)
        .map_err(|e| format!("添加 BasicConstraints 失败: {}", e))?;

    // Key Usage: keyCertSign, cRLSign
    let ku = KeyUsage::new()
        .key_cert_sign()
        .crl_sign()
        .build()
        .map_err(|e| format!("KeyUsage 构建失败: {}", e))?;
    builder
        .append_extension(ku)
        .map_err(|e| format!("添加 KeyUsage 失败: {}", e))?;

    // Subject Key Identifier
    let ctx = builder.x509v3_context(None, None);
    let ski = SubjectKeyIdentifier::new()
        .build(&ctx)
        .map_err(|e| format!("SubjectKeyIdentifier 构建失败: {}", e))?;
    builder
        .append_extension(ski)
        .map_err(|e| format!("添加 SubjectKeyIdentifier 失败: {}", e))?;

    builder
        .sign(&key_pair, MessageDigest::sha256())
        .map_err(|e| format!("CA 证书签名失败: {}", e))?;

    let cert = builder.build();
    let cert_pem = String::from_utf8(cert.to_pem().map_err(|e| format!("PEM 编码失败: {}", e))?)
        .map_err(|e| format!("UTF-8 转换失败: {}", e))?;
    let cert_der = cert.to_der().map_err(|e| format!("DER 编码失败: {}", e))?;
    let key_pem =
        String::from_utf8(key_pair.private_key_to_pem_pkcs8().map_err(|e| format!("私钥 PEM 编码失败: {}", e))?)
            .map_err(|e| format!("UTF-8 转换失败: {}", e))?;

    let subject_str = subject_to_string(&params.subject);
    let alg_name = algorithm_name(&params.key_algorithm);
    let sha256_fp = compute_sha256_fingerprint(&cert_der);
    let sha1_fp = compute_sha1_fingerprint(&cert_der);

    Ok(CertInfo {
        cert_pem,
        key_pem,
        subject: subject_str.clone(),
        issuer: Some(subject_str),
        serial_number: serial_hex,
        valid_from: format_utc_now(),
        valid_to: format_utc_after(params.validity_days),
        sha256_fingerprint: sha256_fp,
        sha1_fingerprint: sha1_fp,
        key_algorithm: alg_name,
        san: None,
    })
}

/// Generate an SSL certificate signed by a CA
pub fn generate_ssl(
    params: &SslParams,
    ca_cert_pem: &str,
    ca_key_pem: &str,
) -> Result<CertInfo, String> {
    let key_pair = generate_pkey(&params.key_algorithm)?;
    let subject_name = build_subject_name(&params.subject)?;

    let ca_cert = X509::from_pem(ca_cert_pem.as_bytes())
        .map_err(|e| format!("CA 证书解析失败: {}", e))?;
    let ca_key = PKey::private_key_from_pem(ca_key_pem.as_bytes())
        .map_err(|e| format!("CA 私钥解析失败: {}", e))?;

    let mut builder =
        X509Builder::new().map_err(|e| format!("X509Builder 创建失败: {}", e))?;

    builder
        .set_version(2)
        .map_err(|e| format!("设置版本失败: {}", e))?;

    let (bn, serial_hex) = generate_serial()?;
    let asn1_int = bn.to_asn1_integer().map_err(|e| format!("序列号转换失败: {}", e))?;
    builder
        .set_serial_number(&asn1_int)
        .map_err(|e| format!("设置序列号失败: {}", e))?;

    builder
        .set_subject_name(&subject_name)
        .map_err(|e| format!("设置主题失败: {}", e))?;
    builder
        .set_issuer_name(ca_cert.subject_name())
        .map_err(|e| format!("设置签发者失败: {}", e))?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|e| format!("设置生效时间失败: {}", e))?;
    let not_after = Asn1Time::days_from_now(params.validity_days as u32)
        .map_err(|e| format!("设置过期时间失败: {}", e))?;
    builder
        .set_not_before(&not_before)
        .map_err(|e| format!("设置 not_before 失败: {}", e))?;
    builder
        .set_not_after(&not_after)
        .map_err(|e| format!("设置 not_after 失败: {}", e))?;

    builder
        .set_pubkey(&key_pair)
        .map_err(|e| format!("设置公钥失败: {}", e))?;

    // Basic Constraints: CA:FALSE
    let bc = BasicConstraints::new()
        .build()
        .map_err(|e| format!("BasicConstraints 构建失败: {}", e))?;
    builder
        .append_extension(bc)
        .map_err(|e| format!("添加 BasicConstraints 失败: {}", e))?;

    // Key Usage: digitalSignature, keyEncipherment
    let ku = KeyUsage::new()
        .digital_signature()
        .key_encipherment()
        .build()
        .map_err(|e| format!("KeyUsage 构建失败: {}", e))?;
    builder
        .append_extension(ku)
        .map_err(|e| format!("添加 KeyUsage 失败: {}", e))?;

    // Extended Key Usage: serverAuth
    let eku = ExtendedKeyUsage::new()
        .server_auth()
        .build()
        .map_err(|e| format!("ExtendedKeyUsage 构建失败: {}", e))?;
    builder
        .append_extension(eku)
        .map_err(|e| format!("添加 ExtendedKeyUsage 失败: {}", e))?;

    // Subject Alternative Name
    let ctx = builder.x509v3_context(Some(&ca_cert), None);
    let mut san_builder = SubjectAlternativeName::new();
    let mut san_names = Vec::new();
    for dns in &params.san.dns_names {
        if !dns.is_empty() {
            san_builder.dns(dns);
            san_names.push(dns.clone());
        }
    }
    for ip in &params.san.ip_addresses {
        if !ip.is_empty() {
            san_builder.ip(ip);
            san_names.push(format!("IP:{}", ip));
        }
    }
    let san_ext = san_builder
        .build(&ctx)
        .map_err(|e| format!("SAN 构建失败: {}", e))?;
    builder
        .append_extension(san_ext)
        .map_err(|e| format!("添加 SAN 失败: {}", e))?;

    // Authority Key Identifier
    let aki_ctx = builder.x509v3_context(Some(&ca_cert), None);
    let aki = AuthorityKeyIdentifier::new()
        .keyid(true)
        .build(&aki_ctx)
        .map_err(|e| format!("AuthorityKeyIdentifier 构建失败: {}", e))?;
    builder
        .append_extension(aki)
        .map_err(|e| format!("添加 AuthorityKeyIdentifier 失败: {}", e))?;

    builder
        .sign(&ca_key, MessageDigest::sha256())
        .map_err(|e| format!("SSL 证书签名失败: {}", e))?;

    let cert = builder.build();
    let cert_pem = String::from_utf8(cert.to_pem().map_err(|e| format!("PEM 编码失败: {}", e))?)
        .map_err(|e| format!("UTF-8 转换失败: {}", e))?;
    let cert_der = cert.to_der().map_err(|e| format!("DER 编码失败: {}", e))?;
    let key_pem =
        String::from_utf8(key_pair.private_key_to_pem_pkcs8().map_err(|e| format!("私钥 PEM 编码失败: {}", e))?)
            .map_err(|e| format!("UTF-8 转换失败: {}", e))?;

    let subject_str = subject_to_string(&params.subject);
    let ca_subject = x509_name_to_string(ca_cert.subject_name());
    let alg_name = algorithm_name(&params.key_algorithm);
    let sha256_fp = compute_sha256_fingerprint(&cert_der);
    let sha1_fp = compute_sha1_fingerprint(&cert_der);

    Ok(CertInfo {
        cert_pem,
        key_pem,
        subject: subject_str,
        issuer: Some(ca_subject),
        serial_number: serial_hex,
        valid_from: format_utc_now(),
        valid_to: format_utc_after(params.validity_days),
        sha256_fingerprint: sha256_fp,
        sha1_fingerprint: sha1_fp,
        key_algorithm: alg_name,
        san: Some(san_names),
    })
}

/// Parse a PEM certificate and return its info (for import preview)
pub fn parse_certificate(cert_pem: &str) -> Result<CertInfo, String> {
    let cert = X509::from_pem(cert_pem.as_bytes())
        .map_err(|e| format!("证书解析失败: {}", e))?;

    let der = cert.to_der().map_err(|e| format!("DER 编码失败: {}", e))?;
    let sha256_fp = compute_sha256_fingerprint(&der);
    let sha1_fp = compute_sha1_fingerprint(&der);

    let subject = x509_name_to_string(cert.subject_name());

    Ok(CertInfo {
        cert_pem: cert_pem.to_string(),
        key_pem: String::new(),
        subject,
        issuer: None,
        serial_number: String::new(),
        valid_from: String::new(),
        valid_to: String::new(),
        sha256_fingerprint: sha256_fp,
        sha1_fingerprint: sha1_fp,
        key_algorithm: String::new(),
        san: None,
    })
}