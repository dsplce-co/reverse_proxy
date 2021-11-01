# Reverse Proxy with TLS

Simply wraps an existing HTTP service to make it HTTPS.
By default, assumes your service is on port 8080 and serves it at port 4343.

```
$ cargo install reverse_proxy

  <set up TLS cert directory>

$ reverse_proxy
```

## Generating certificates

[Example setup from Rustls](https://github.com/rustls/rustls/blob/main/test-ca/build-a-pki.sh)

### How to bind the real SSL port

```
$ sudo setcap CAP_NET_BIND_SERVICE=+eip <binary>
$ PORT=443 <binary>
```
