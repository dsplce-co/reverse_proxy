FROM rust:1 AS builder
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

WORKDIR /opt
COPY . .
RUN cargo install --path .

FROM ubuntu AS runtime
COPY --from=builder /.cargo/bin/reverse_proxy /opt

EXPOSE 4343

WORKDIR /opt
ENTRYPOINT [ "/opt/reverse_proxy" ]



