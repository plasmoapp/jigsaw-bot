# Rust as the base image
FROM rustlang/rust:nightly as build_backend

# Create a new empty shell project
RUN USER=root cargo new --bin jigsaw-backend
WORKDIR jigsaw-backend

# Copy common module
COPY ./jigsaw-common ../jigsaw-common

# Copy cargo files
COPY ./jigsaw-backend/Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./jigsaw-backend/src ./src

# Build for release.
RUN rm ./target/release/deps/jigsaw_backend*
RUN cargo build --release

# Game image
FROM barichello/godot-ci:3.5.2 as build_game

WORKDIR jigsaw-game

COPY ./jigsaw-game ./

RUN mkdir export
RUN godot -v --export "HTML5" ./export/index.html

# The final base image
FROM debian:bullseye-slim

# Setup workdir
WORKDIR /jigsaw-backend/

# Copy from the previous build
COPY --from=build_backend /jigsaw-backend/target/release/jigsaw-backend .
COPY --from=build_game /jigsaw-game/export ./public

# Run the binary
CMD ["./jigsaw-backend"]
