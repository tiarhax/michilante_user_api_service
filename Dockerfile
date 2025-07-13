FROM ubuntu:24.04 AS chef
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y 
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cargo-chef 

WORKDIR app


FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release


FROM ubuntu:24.04 AS runtime
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    openssl \
    ca-certificates \
    pkg-config \
    curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
WORKDIR app

    
COPY --from=builder /app/target/release/user_api_server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/user_api_server"]

