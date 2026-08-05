# Database

## Overview

SQLite database with Diesel ORM for Rust.

## Migrations

Migrations are in `src-core/migrations/`.

Run migrations:
```bash
diesel migration run
```

Create new migration:
```bash
diesel migration generate <name>
```

## Schema

The schema is auto-generated in `src-core/src/schema.rs`.
