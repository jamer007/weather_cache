FROM rust:1.75-bookworm
# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/weather_cache"]
