#!/usr/bin/env bash
# ============================================================
# 一键生成内部 CA，并为 hot-sftp.it.local 签发带 SAN 的证书
# 用法: ./generate.sh
# 输出:
#   certs/ca.crt                  根 CA（需安装到所有客户端信任库）
#   certs/hot-sftp-server.crt     服务器证书
#   certs/hot-sftp-fullchain.crt  完整链（nginx ssl_certificate 用）
#   private/hot-sftp-server.key   服务器私钥（nginx ssl_certificate_key 用）
# ============================================================
set -euo pipefail
cd "$(dirname "$0")"

SERVER_CN="hot-sftp.it.local"
SERVER_IP="10.61.254.188"
CA_DAYS=3650
SERVER_DAYS=730

mkdir -p certs private newcerts
chmod 700 private
touch index.txt
[ -s serial ] || openssl rand -hex 16 > serial

# ---------- 1. 根 CA ----------
if [ ! -f private/ca.key ]; then
    echo "==> 生成 CA 私钥 (RSA 4096)"
    openssl genrsa -out private/ca.key 4096
fi
chmod 600 private/ca.key

if [ ! -f certs/ca.crt ]; then
    echo "==> 自签根证书 CN=it-LOCAL-CA-1 (${CA_DAYS} 天)"
    openssl req -new -x509 -key private/ca.key -out certs/ca.crt \
        -days "$CA_DAYS" -sha256 -config openssl.cnf -extensions v3_ca \
        -subj "/C=CN/O=IT/CN=it-LOCAL-CA-1"
fi

# ---------- 2. 服务器私钥 + CSR ----------
echo "==> 生成服务器私钥 (RSA 2048)"
openssl genrsa -out private/hot-sftp-server.key 2048
chmod 600 private/hot-sftp-server.key

echo "==> 生成 CSR (SAN: DNS:${SERVER_CN}, IP:${SERVER_IP})"
openssl req -new -key private/hot-sftp-server.key -out hot-sftp-server.csr \
    -config openssl.cnf -extensions v3_req \
    -subj "/C=CN/O=IT/OU=SFTP/CN=${SERVER_CN}"

# ---------- 3. 签发服务器证书 ----------
echo "==> 用 CA 签发服务器证书 (${SERVER_DAYS} 天)"
openssl ca -config openssl.cnf -batch -notext \
    -in hot-sftp-server.csr -out certs/hot-sftp-server.crt \
    -days "$SERVER_DAYS" -extensions v3_server \
    -keyfile private/ca.key -cert certs/ca.crt

# ---------- 4. 拼接完整链 ----------
cat certs/hot-sftp-server.crt certs/ca.crt > certs/hot-sftp-fullchain.crt

# ---------- 5. 清理 ----------
rm -f hot-sftp-server.csr

# ---------- 6. 验证 ----------
echo
echo "==> 证书链校验:"
openssl verify -CAfile certs/ca.crt certs/hot-sftp-server.crt

echo
echo "==> 服务器证书信息:"
openssl x509 -in certs/hot-sftp-server.crt -noout \
    -subject -issuer -dates -ext subjectAltName

echo
echo "==> 输出文件:"
ls -l certs/*.crt private/*.key
