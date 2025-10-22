# syntax=docker/dockerfile:1

# Define Build environment
FROM rust AS builder
# Define where the stuff be happening
WORKDIR /usr/src/app

# Make the dependencies
# COPY Cargo.toml Cargo.lock ./
#make minimal rust project
# RUN mkdir src && echo "fn main() {}" > src/main.rs
# RUN cargo build --release
# RUN rm -rf src

#Main code
COPY . .

RUN cargo build --release

# Now make the runtime release version (much smaller)
FROM rust:slim

COPY --from=builder /usr/src/app/target/release/swgoh-utils-api /usr/local/bin/swgoh-utils-api

# RUN useradd -m appuser
# USER appuser

CMD ["swgoh-utils-api"]
# CMD ["sleep", "infinity"]

EXPOSE 7474