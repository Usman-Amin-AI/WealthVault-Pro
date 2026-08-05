# Commands

Backend command wrappers for Tauri and Web modes.

## Purpose

Provides unified API for both desktop (Tauri) and web deployments.

## Pattern

Each command checks RUN_ENV and routes to appropriate backend.

## Guidelines

- Support both Tauri and Web modes
- Handle errors consistently
- Log operations for debugging
