# Reverse Proxy with TLS

Simply wraps an existing HTTP service to make it HTTPS.
By default, assumes your service is on port 8080 and serves it at port 4343.

## Generating certificates

```
$ cd tls
$ ./build-cert.sh
```

Then (on MacOS) drag `tls/ecdsa/ca.cert` into your login keychain and set it to trusted.

## Operation

Return to this directory and `cargo run` to start proxying.

Start your HTTP service on port 8080 and see if it appears at: https://localhost:4343/

### How to bind the real SSL port

```
$ sudo setcap CAP_NET_BIND_SERVICE=+eip <binary>
$ PORT=443 <binary>
```
