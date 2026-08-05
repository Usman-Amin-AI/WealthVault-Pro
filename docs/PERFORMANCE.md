# Performance Guidelines

## Frontend

- Use React.memo for expensive components
- Virtualize long lists
- Lazy load routes and heavy components
- Use TanStack Query for caching

## Backend

- Use database indexes
- Batch operations where possible
- Cache frequently accessed data
- Profile with cargo flamegraph
