FROM rust:1.76-slim as planner
WORKDIR app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.76-slim as cacher
WORKDIR app
ENV OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu"
ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl"
RUN apt-get update && apt-get install -y build-essential wget pkg-config libssl-dev libpq-dev zlib1g-dev ca-certificates && update-ca-certificates
# Build libssl-1.1 from source
RUN wget https://www.openssl.org/source/openssl-1.1.1.tar.gz \
    && tar -xzvf openssl-1.1.1.tar.gz \
    && cd openssl-1.1.1 \
    && ./config --prefix=/usr/local/ssl --openssldir=/usr/local/ssl shared zlib \
    && make \
    && make install
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.76-slim as builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/ssl/lib/* /usr/local/lib/
COPY --from=wcsiu/tdlib:1.8.0 /td/build/libtd* /usr/local/lib/
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH
RUN cargo build --release --verbose

FROM rust:1.76-slim as runtime
WORKDIR app
COPY --from=builder /usr/local/lib/* /usr/local/lib/
COPY --from=builder /app/target/release/telemap /usr/local/bin
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

ENTRYPOINT ["telemap"]
CMD ["--help"]