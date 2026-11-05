FROM rust:1.85.1@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY mcp ./mcp
COPY dashboard ./dashboard
RUN cargo build --locked --bin ariadne_mcp_backend
VOLUME ["/data"]
EXPOSE 8080 8090
CMD ["python3", "dashboard/supervisor.py"]
