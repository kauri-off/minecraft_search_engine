FROM rust

WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {println!(\"dummy\")}" > src/main.rs
RUN cargo build --release

RUN rm -rf src
COPY ./src ./src
COPY ./minecraft_protocol ./minecraft_protocol

RUN cargo build --release

CMD [ "./target/release/minecraft_search_engine" ]
