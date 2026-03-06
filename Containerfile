# =============================================================================
# Rule 30 VDF — Rootless Podman Build (daemonless)
# podman build -t rule30-vdf -f Containerfile .
# podman run --rm rule30-vdf
# =============================================================================
FROM docker.io/library/rust:1.77-slim AS builder

WORKDIR /build
COPY Cargo.toml ./
COPY src/ ./src/

RUN cargo build --release

# -- Runtime stage (minimal) --------------------------------------------------
FROM docker.io/library/debian:bookworm-slim

COPY --from=builder /build/target/release/rule30-vdf /usr/local/bin/rule30-vdf

ENV RULE30_CELLS=64
ENV RULE30_STEPS=1024

ENTRYPOINT ["rule30-vdf"]
