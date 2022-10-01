FROM ubuntu:22.04 as chef
RUN apt update && apt upgrade -y
RUN apt-get install -y pkg-config libssl-dev curl build-essential
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH $PATH:/root/.cargo/bin
RUN cargo install cargo-chef
ENV CARGO_TARGET_DIR=/home/rust/target
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!

FROM ubuntu:22.04 AS runtime
RUN apt update && apt upgrade -y
RUN apt-get install -y libssl-dev
COPY --from=builder /home/rust/target/release/my-client /usr/local/bin/my-client
ENTRYPOINT ["my-client"]
