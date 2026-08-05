# Deployment and Onboarding Guide

## Purpose

This guide explains how to set up WealthVault Pro for local development, test
runs, and deployment-oriented validation. It is intended for engineers,
maintainers, and team members who need to get the app running quickly while
preserving the repo’s expected structure.

## Prerequisites

Before running the project, install:

- Node.js 20+
- pnpm
- Rust and Cargo
- Platform-specific Tauri toolchain requirements

## Local Setup

```bash
pnpm install
```

### Option 1: Frontend-only local development

```bash
pnpm dev
```

### Option 2: Web/server mode

```bash
pnpm run dev:web
```

### Option 3: Desktop app development

```bash
pnpm tauri dev
```

## Production Build Validation

### Frontend and workspace build

```bash
pnpm build
```

### Desktop packaging validation

```bash
pnpm tauri build
```

## Recommended Validation Flow

1. Run the frontend build.
2. Confirm the Rust server/web mode still boots correctly.
3. Confirm the Tauri desktop shell starts and packages cleanly.
4. Validate that local settings and secrets stay within the product’s secure
   design.

## Environment Notes

- Use the repo’s existing environment examples where applicable.
- Respect the local-first storage model.
- Keep output folders and generated build artifacts out of tracked source logic.

## Team Onboarding Checklist

- Clone the repository
- Install Node.js, pnpm, Rust, and Tauri prerequisites
- Run `pnpm install`
- Confirm local development mode works
- Confirm a production build path works for the target platform
- Review the architecture docs and security guardrails before modifying core
  flows

## Recommended Troubleshooting Order

1. Confirm package installation succeeded.
2. Confirm the workspace dependencies are present.
3. Confirm the target runtime mode is the correct one for the change.
4. If desktop packaging fails, verify native toolchain availability and local
   build space.

## Operational Notes

This repository is a multi-runtime project. Changes that impact shared flows
should be validated in both the desktop path and the web/server path when
practical.
