

FROM rust:latest AS build

RUN USER=root cargo new --bin frontend

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools

WORKDIR /frontend

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --target=x86_64-unknown-linux-musl --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./templates ./templates
COPY ./static ./static

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/info_taulu*

RUN cargo build --target=x86_64-unknown-linux-musl --release

FROM rust:latest

EXPOSE 3060
ENV TZ="Europe/Helsinki"

COPY --from=build /frontend/target/x86_64-unknown-linux-musl/release/info-taulu .
COPY --from=build /frontend/src /frontend/src
COPY --from=build /frontend/static /frontend/static
COPY --from=build /frontend/templates /frontend/templates

CMD ["./info-taulu"]