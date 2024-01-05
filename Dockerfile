FROM rust:1.75-bookworm
COPY src/ src/
COPY Cargo.toml ./
COPY Cargo.lock ./

RUN cargo build --release

CMD ["./target/release/weather_cache"]
