# Internal Product Documentation

## Product Summary

WealthVault Pro is a private, local-first investment portfolio management
application. It is built around account tracking, portfolio analytics,
market-data sync, and configurable integrations. The product is designed to run
as a desktop-first native app while also supporting a browser/web execution mode
through server-side routing.

## Product Goals

1. Help users track portfolio value and holdings across accounts.
2. Provide clear insights into investment composition, balance, and performance.
3. Support workflow automation for imports, broker-sync settings, and
   market-data backfills.
4. Keep sensitive financial data located locally and secure by default.
5. Enable extension through an addon architecture with permissions and runtime
   isolation.

## Product Architecture

### Frontend Layer

The frontend is a React + TypeScript application with Vite, routing, and a
modern component system. It is responsible for the user interface, dashboards,
settings flows, charting, and orchestration of commands into the underlying
runtime.

### Desktop Runtime Layer

Tauri provides the native application shell and OS integration. This layer is
the main runtime for packaged desktop delivery.

### Server/Web Runtime Layer

Axum runs an optional HTTP server and web-facing route layer. This allows the
same product to be surfaced through a web-compatible mode when needed.

### Core Business Layer

The Rust core layer contains shared services and repositories that back
portfolio, account, market, and configuration workflows. This is where the
shared business logic should live.

## Key User Journeys

### Onboarding

- Create or import account data
- Define the user’s base settings and portfolio configuration
- Start portfolio synchronization or import flows

### Daily Portfolio Review

- View total portfolio value
- Inspect positions, allocations, and recent activity
- Review account performance summaries

### Market Data Maintenance

- Configure market data providers
- Import quote history when needed
- Recalculate portfolio data after data refresh

### Addon Enablement

- Install or develop addons with the provided SDK/dev tools
- Expose only required permissions and runtime capabilities

## Development Conventions

- Keep frontend UI logic thin and declarative.
- Keep business rules in shared core services when possible.
- Prefer existing adapters and command wrappers over ad hoc IO calls.
- Keep permissions and data access scoped tightly.
- Maintain local-first behavior and avoid cloud persistence for core user data.

## Teams / Ownership Model

- Product and UX: dashboard, settings flow, user experience polish
- Frontend engineering: React routes, pages, components, hooks
- Rust/core engineering: repositories, calculations, data services
- Desktop integration: Tauri shell, packaging, OS integration
- Platform/security: secrets handling, permissions, data protection

## Release Readiness Checklist

- Frontend build passes
- Desktop build path compiles and packages successfully
- Server/web mode remains stable
- Sensitive settings and secrets remain secure
- Addon permissions remain appropriately constrained

## Success Criteria

The product is successful when a user can reliably:

- open the app on desktop
- review portfolio value and holdings
- import or sync data sources
- understand the portfolio from charts and summary screens
- trust that the app preserves privacy and local data ownership
