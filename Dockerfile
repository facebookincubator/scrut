#
# Build stage
#
FROM rust:1-slim-bookworm AS builder
ARG SKIP_TESTS=no

ENV BUILD_ARGS="--bin scrut --release"

RUN apt update && \
    apt install -y build-essential && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

# Cache build dependencies separately
WORKDIR /app
RUN mkdir -p src/bin && \
    printf 'extern crate scrut;\n\ninclude!(concat!(env!("OUT_DIR"), "/version.rs"));\n\nfn main() {}\n' > src/bin/main.rs && \
    echo > src/lib.rs
COPY Cargo.toml build.rs .
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build $BUILD_ARGS --features volatile_tests && \
    rm -rf target/release

# Build the binary
RUN rm -rf src/ build.rs Cargo.toml
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build $BUILD_ARGS && \
    ls -lha target/release/ && \
    cat src/bin/main.rs && \
    cp -av target/release/scrut .

# Run tests (conditionally)
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    if [ "$SKIP_TESTS" != "yes" ]; then \
        SCRUT_BIN="$(pwd)"/scrut make cargotest selftest; \
    fi

#
# Run stage
#
FROM debian:bookworm-slim

# Copy previously build binary
WORKDIR /app
COPY --from=builder /app/scrut /usr/local/bin

# Run the container to run scrut
ENTRYPOINT ["scrut"]
