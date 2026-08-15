export enum KeyAlgorithm {
  Rsa2048 = 'Rsa2048',
  Rsa4096 = 'Rsa4096',
  EcdsaP256 = 'EcdsaP256',
  EcdsaP384 = 'EcdsaP384',
}

export interface SubjectInfo {
  common_name: string;
  organization?: string;
  organizational_unit?: string;
  country?: string;
}

export interface CaParams {
  subject: SubjectInfo;
  key_algorithm: KeyAlgorithm;
  validity_days: number;
}

export interface SanEntry {
  dns_names: string[];
  ip_addresses: string[];
}

export interface SslParams {
  subject: SubjectInfo;
  san: SanEntry;
  key_algorithm: KeyAlgorithm;
  validity_days: number;
}

export interface CertInfo {
  cert_pem: string;
  key_pem: string;
  subject: string;
  issuer: string | null;
  serial_number: string;
  valid_from: string;
  valid_to: string;
  sha256_fingerprint: string;
  sha1_fingerprint: string;
  key_algorithm: string;
  san: string[] | null;
}

export interface DomainCertResult {
  cert_info: CertInfo;      // 证书信息的嵌套对象
  pfx_buffer: number[];     // 二进制 PFX 数据（由 Vec<u8> 序列化）
  ca_cert_pem: string;     // 独立的 CA.crt 内容
}