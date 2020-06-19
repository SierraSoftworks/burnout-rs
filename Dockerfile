FROM rust:1

WORKDIR /src

# Pre-build all dependencies
RUN USER=root cargo init --bin --name burnout
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN cargo build --release
RUN rm src/*.rs
RUN rm ./target/release/deps/burnout*

# Add the source code
COPY . .

# Run the test suite
RUN cargo test --release
RUN rm /src/target/release/deps/burnout*


# Build the rest of the project
RUN cargo build --release --bin burnout --features "table_storage"

# Ensure that the binary is at a known location for the next stage
RUN rm /src/target/release/deps/burnout*.d
RUN cp /src/target/release/deps/burnout* /src/target/release/deps/burnout

FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies

RUN apt-get update && apt-get install -y libssl1.1 ca-certificates

COPY --from=0 /src/target/release/deps/burnout /app/burnout

WORKDIR /app
CMD [ "/app/burnout" ]