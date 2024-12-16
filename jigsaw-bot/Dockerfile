# Rust as the base image
FROM rustlang/rust:nightly as build
ARG BUILD_MODE=release

# Create a new empty shell project
RUN USER=root cargo new --bin jigsaw-bot
WORKDIR jigsaw-bot

# Copy common module
COPY ./jigsaw-common ../jigsaw-common

# Copy cargo files
COPY ./jigsaw-bot/Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release ; else cargo build ; fi
RUN rm src/*.rs

# Copy the source code
COPY ./jigsaw-bot/src ./src

# Build for release.
RUN rm ./target/$BUILD_MODE/deps/jigsaw_bot*
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release ; else cargo build ; fi

# The final base image
FROM debian:bookworm-slim
ARG BUILD_MODE=release

# Install & update ca-certificates
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates

# Setup workdir
WORKDIR /jigsaw-bot/

# Copy from the previous build
COPY --from=build /jigsaw-bot/target/$BUILD_MODE/jigsaw-bot .

# Run the binary
CMD ["./jigsaw-bot"]
