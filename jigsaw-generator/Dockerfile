# Rust as the base image
FROM rustlang/rust:nightly as build
ARG BUILD_MODE=release

# Create a new empty shell project
RUN USER=root cargo new --bin jigsaw-generator
WORKDIR jigsaw-generator

# Copy common module
COPY ./jigsaw-common ../jigsaw-common

# Copy cargo files
COPY ./jigsaw-generator/Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release ; else cargo build ; fi
RUN rm src/*.rs

# Copy the source code
COPY ./jigsaw-generator/src ./src

# Build for release.
RUN rm ./target/$BUILD_MODE/deps/jigsaw_generator*
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release ; else cargo build ; fi

# The final base image
FROM debian:bookworm-slim
ARG BUILD_MODE=release

# Setup workdir
WORKDIR /jigsaw-generator/

# Copy from the previous build
COPY --from=build /jigsaw-generator/target/$BUILD_MODE/jigsaw-generator .

# Run the binary
CMD ["./jigsaw-generator"]
