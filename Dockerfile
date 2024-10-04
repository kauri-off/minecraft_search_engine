FROM rust

WORKDIR /app

RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml .
COPY minecraft_protocol minecraft_protocol 

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY src src

RUN cargo build --release

CMD ["target/release/mse"]