# Deployment Guide

## Docker

```bash
docker build -t investwise .
docker run -p 8080:8080 investwise
```

## Manual Deployment

1. Build the application: `pnpm build`
2. Run the server: `cargo run --manifest-path src-server/Cargo.toml`
3. Access at `http://localhost:8080`
