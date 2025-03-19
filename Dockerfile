# Use Cargo Chef for dependency caching and build stages
FROM lukemathwalker/cargo-chef:latest-rust-1.82.0 as chef
WORKDIR /app
RUN apt update && apt install -y lld clang

# Planner stage for caching dependencies
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage for building the application
FROM chef as builder
WORKDIR /app

# Copy the dependency recipe for cargo-chef
COPY --from=planner /app/recipe.json recipe.json

# Build all dependencies based on the recipe for caching
RUN cargo chef cook --release --recipe-path recipe.json

# Copy application source code, including the `twilio` directory
COPY . .
COPY ./twilio ./twilio
COPY ./static ./static

RUN cargo install sqlx-cli
# ENV DATABASE_URL=
RUN cargo sqlx prepare
# Build the application
RUN cargo build --release

# Final runtime stage
FROM lukemathwalker/cargo-chef:latest-rust-1.82.0 as runtime
WORKDIR /app

# Install only necessary runtime dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates ffmpeg \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user and switch to it
RUN useradd -m appuser
USER appuser

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/gamecall gamecall

# Expose the port your application is listening on (replace 8080 with your actual port)
EXPOSE 8080

# Set the command to run your application
CMD ["./gamecall"]
