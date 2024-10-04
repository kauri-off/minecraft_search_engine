FROM rust

WORKDIR /app

COPY Cargo.toml .
COPY ./minecraft_protocol ./minecraft_protocol

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

RUN rm -rf ./src
COPY ./src ./src

RUN cargo build --release

CMD [ "./target/release/minecraft_search_engine" ]
