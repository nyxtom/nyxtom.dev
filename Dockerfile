FROM rust:1.61.0 as builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release

# copy compiled from builder
FROM rust:1.61.0 as runtime

WORKDIR /app
COPY --from=builder /app/target/release/notes notes

ENV RUST_LOG info
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./notes"]
