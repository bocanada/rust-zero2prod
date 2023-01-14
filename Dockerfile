FROM blackdex/rust-musl:aarch64-musl as muslchef

RUN cargo install cargo-chef

WORKDIR /app

FROM muslchef as planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM muslchef as builder

COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --target aarch64-unknown-linux-musl --recipe-path recipe.json

# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .

ENV SQLX_OFFLINE true

# Build our project
RUN cargo build --release --target aarch64-unknown-linux-musl --bin zero2prod

FROM scratch as runtime

WORKDIR /app

COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/zero2prod zero2prod

COPY .config .config

ENV APP_ENVIRONMENT production

ENTRYPOINT ["./zero2prod"]
