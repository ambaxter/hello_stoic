# 1: Build the exe
FROM rust:1.47 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# 1b: Download and compile Rust dependencies
RUN USER=root cargo new hello_stoic
WORKDIR /usr/src/hello_stoic
COPY Cargo.toml ./
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 1c: Build the exe using actual source code
COPY src ./src
COPY static ./static
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 3: Copy the exe and extra files ("static") to an empty Docker image
FROM rust:1.47
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools
COPY --from=builder /usr/local/cargo/bin/hello_stoic .
COPY static ./static
RUN mkdir /data
#USER 1000
CMD ["./hello_stoic"]
EXPOSE 8080