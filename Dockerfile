FROM rust:latest
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release
ENV RUST_LOG info
ENTRYPOINT ["./target/release/notes"]


