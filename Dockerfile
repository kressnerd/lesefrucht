FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY twins ./twins
COPY xtask ./xtask
COPY tests ./tests
RUN cargo build --release --bin lesefrucht

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends calibre ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lesefrucht /usr/local/bin/lesefrucht
COPY lesefrucht.toml /etc/lesefrucht.toml
EXPOSE 8787
ENTRYPOINT ["lesefrucht", "--config", "/etc/lesefrucht.toml"]
