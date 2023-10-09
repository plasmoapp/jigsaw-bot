# Rust as the base image
FROM rustlang/rust:nightly as build

# Create a new empty shell project
RUN USER=root cargo new --bin jigsaw-bot
WORKDIR jigsaw-bot

# Copy common module
COPY ./jigsaw-common ../jigsaw-common

# Copy cargo files
COPY ./jigsaw-bot/Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./jigsaw-bot/src ./src

# Build for release.
RUN rm ./target/release/deps/jigsaw_bot*
RUN cargo build --release

# The final base image
FROM debian:bullseye-slim

# Install & update ca-certificates
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates

# Setup workdir
WORKDIR /jigsaw-bot/

# Copy from the previous build
COPY --from=build /jigsaw-bot/target/release/jigsaw-bot .

# Run the binary
CMD ["./jigsaw-bot"]
