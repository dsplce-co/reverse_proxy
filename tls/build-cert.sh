#!/bin/sh

# cribbed from: https://github.com/rustls/rustls/blob/main/test-ca/build-a-pki.sh

set -xe

# use the keg-only openssl on MacOS
if [ -d "/usr/local/Cellar/openssl@3/" ]; then
  export PATH=/usr/local/Cellar/openssl@3/3.0.0/bin/:$PATH
fi

rm -rf ecdsa/
mkdir -p ecdsa/

openssl ecparam -name prime256v1 -out ecdsa/nistp256.pem
openssl ecparam -name secp384r1 -out ecdsa/nistp384.pem

# build CA
openssl req -nodes \
          -x509 \
          -newkey ec:ecdsa/nistp384.pem \
          -keyout ecdsa/ca.key \
          -out ecdsa/ca.cert \
          -sha256 \
          -batch \
          -days 3650 \
          -subj "/CN=Self-signed ECDSA CA"

# request a cert
openssl req -nodes \
          -newkey ec:ecdsa/nistp256.pem \
          -keyout ecdsa/end.key \
          -out ecdsa/end.req \
          -sha256 \
          -batch \
          -subj "/CN=dev.local"

# create the cert
openssl x509 -req \
          -in ecdsa/end.req \
          -out ecdsa/end.cert \
          -CA ecdsa/ca.cert \
          -CAkey ecdsa/ca.key \
          -sha256 \
          -days 365 \
          -set_serial 1337 \
          -extensions v3_end -extfile openssl.cnf

cat ecdsa/end.cert ecdsa/ca.cert > ecdsa/end.fullchain
