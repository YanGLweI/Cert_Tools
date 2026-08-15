mod certgen;

use certgen::{CaParams, DomainCertParams, CertInfo, DomainCertResult, SslParams};

#[tauri::command]
fn generate_ca(params: CaParams) -> Result<CertInfo, String> {
    certgen::generate_ca(&params)
}

#[tauri::command]
fn generate_ssl(params: SslParams, ca_cert_pem: String, ca_key_pem: String) -> Result<CertInfo, String> {
    certgen::generate_ssl(&params, &ca_cert_pem, &ca_key_pem)
}

#[tauri::command]
fn parse_certificate(cert_pem: String) -> Result<CertInfo, String> {
    certgen::parse_certificate(&cert_pem)
}

#[tauri::command]
fn generate_domain_certificate(params: DomainCertParams, pfx_password: String) -> Result<DomainCertResult, String> {
    certgen::generate_domain_certificate(&params, &pfx_password)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            generate_ca,
            generate_ssl,
            parse_certificate,
            generate_domain_certificate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}