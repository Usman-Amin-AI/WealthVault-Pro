# WealthVault Pro

<p align="center">
  <img src="./app-icon.png" alt="WealthVault Pro app icon" width="180" />
</p>

<p align="center">
  <strong>Private, local-first portfolio intelligence for modern investors.</strong>
</p>

WealthVault Pro is a desktop-first investment portfolio platform built with
React, Vite, Tauri, and Rust. It is designed for people who want to track
holdings, monitor performance, manage account data, and sync market information
from a secure local workspace.

## Why WealthVault Pro

- Local-first portfolio tracking with strong privacy assumptions
- Desktop-native UX via Tauri and a React frontend
- Portfolio analytics, valuations, account summaries, and activity workflows
- Broker and market-data integration hooks
- Extensible addon architecture for custom dashboards and tooling

## Features

- Portfolio overview and holdings analysis
- Performance tracking and account-level metrics
- Market-data provider configuration and sync workflows
- Broker/account connection settings
- Addon SDK and runtime support for modular extensions
- Cross-runtime command access between desktop and web/server modes

## Architecture

WealthVault Pro is organized as a multi-layer workspace:

- Frontend: React + TypeScript + Vite
- Desktop shell: Tauri + Rust
- Server/web mode: Axum + Rust
- Core services: Rust logic, repositories, and persistence
- Shared packages: UI and addon tooling under the monorepo

## Repository Structure

```text
src/           # React application UI
src-core/      # Rust business logic and database services
src-server/    # Axum web/server layer
src-tauri/     # Tauri desktop shell and native integration
packages/      # Shared UI and addon-facing packages
addons/        # Extensible addon packages
public/        # Static assets and images
```

## Quick Start

### Prerequisites

- Node.js 20+
- pnpm
- Rust and Cargo
- Platform Tauri dependencies for your OS

### Install

```bash
pnpm install
```

### Run locally

```bash
pnpm dev
pnpm run dev:web
pnpm tauri dev
```

### Build

```bash
pnpm build
pnpm tauri build
```

## Testing

```bash
pnpm test
pnpm test:coverage
pnpm lint
pnpm type-check
```

## Security and Privacy

- Local-first storage model
- Secrets should stay in OS-backed secure storage where possible
- Addons should request only the minimum permissions they require

## Contributing

Contributions are welcome. Keep changes focused, match the surrounding code
patterns, and validate with the repo’s existing build and test scripts before
submitting.

## License

This project uses the AGPL-3.0 license as defined in the repository metadata.
