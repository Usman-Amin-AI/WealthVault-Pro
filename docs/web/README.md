# InvestWise Web Deployment

## Running locally

```bash
pnpm build
cargo run --manifest-path src-server/Cargo.toml
```

The server listens on `WF_LISTEN_ADDR` (default `0.0.0.0:8080`).

## Docker

To build the container image:

```bash
docker build -t investwise-web .
```

Run the image:

```bash
docker run -p 8080:8080 -v $(pwd)/data:/data investwise-web
```

Frontend assets are served from `/` and API available under `/api/v1`.
# Development Environment Setup

This project has been set up with:
- Node.js and pnpm for package management
- Rust toolchain for the backend server
- Vite for frontend development

## Quick Start

```bash
# Install dependencies
pnpm install

# Run frontend only
pnpm dev

# Run full web mode (frontend + backend)
pnpm run dev:web

# Run tests
pnpm test
```

## Testing

The project uses Vitest for unit testing. All tests should pass before committing.

### Test Files Location
- Frontend tests: `src/**/*.test.ts`
- Test utilities: `src/test/`

### Running Tests
```bash
pnpm test           # Run tests once
pnpm test:watch     # Watch mode
pnpm test:coverage  # With coverage
```

## Troubleshooting

### Common Issues

1. **Port 1420 already in use**
   ```bash
   lsof -i :1420
   kill -9 <PID>
   ```

2. **Rust backend not starting**
   - Ensure Rust is installed: `rustc --version`
   - Check cargo is in PATH: `which cargo`

3. **Dependencies issues**
   ```bash
   rm -rf node_modules pnpm-lock.yaml
   pnpm install
   ```

## Architecture Overview

The project follows a clean architecture with:

- **Frontend**: React + Vite + Tailwind CSS
- **Backend**: Rust with Axum (web) / Tauri (desktop)
- **Database**: SQLite (local storage)
- **State Management**: TanStack Query

### Key Directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Frontend React application |
| `src-core/` | Rust business logic |
| `src-server/` | Axum HTTP server |
| `src-tauri/` | Tauri desktop app |
| `packages/` | Shared packages |

