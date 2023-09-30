####################################################################################################
## Planner Image
####################################################################################################
FROM rust:1.71 as planner
WORKDIR /shellcove
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


####################################################################################################
## Cacher Image
####################################################################################################
FROM rust:1.71 as cacher
WORKDIR /shellcove
RUN cargo install cargo-chef
COPY --from=planner /shellcove/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


####################################################################################################
## Builder
####################################################################################################
FROM rust:1.71 as builder
WORKDIR /shellcove
COPY . .

#copy dependencies
COPY --from=cacher /shellcove/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
# FROM gcr.io/distroless/cc
FROM frolvlad/alpine-glibc
WORKDIR /shellcove

COPY --from=builder /shellcove/target/release/shellcove .
COPY conf/default.toml conf/

# default listening port
EXPOSE 8080

CMD ["./shellcove"]
