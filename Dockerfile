FROM rust:1.63-slim as planner
WORKDIR app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.63-slim as cacher
WORKDIR app
ENV OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu"
ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl"
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.63-slim as builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=wcsiu/tdlib:1.8.0 /td/build/libtd* /usr/local/lib/
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev
RUN RUSTFLAGS="-C link-args=-Wl,-rpath,/usr/local/lib" cargo build --release

FROM rust:1.63-slim as runtime
WORKDIR app
COPY --from=wcsiu/tdlib:1.8.0 /td/build/libtd* /usr/local/lib/
COPY --from=builder /app/target/release/telemap /usr/local/bin

ENTRYPOINT ["telemap"]
CMD ["--help"]