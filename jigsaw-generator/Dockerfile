# Rust as the base image
FROM rustlang/rust:nightly as build

# Create a new empty shell project
RUN USER=root cargo new --bin jigsaw-generator
WORKDIR jigsaw-generator

# Copy common module
COPY ./jigsaw-common ../jigsaw-common

# Copy cargo files
COPY ./jigsaw-generator/Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./jigsaw-generator/src ./src

# Build for release.
RUN rm ./target/release/deps/jigsaw_generator*
RUN cargo build --release

# The final base image
FROM debian:bullseye-slim

# Setup workdir
WORKDIR /jigsaw-generator/

# Copy from the previous build
COPY --from=build /jigsaw-generator/target/release/jigsaw-generator .

# Run the binary
CMD ["./jigsaw-generator"]
