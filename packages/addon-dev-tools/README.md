# @investwise/addon-dev-tools

Development tools for InvestWise addons including hot reload server and CLI.

## Installation

```bash
npm install -g @investwise/addon-dev-tools
```

## CLI Commands

### Create New Addon

```bash
investwise create my-awesome-addon
```

### Start Development Server

```bash
# In your addon directory
investwise dev
```

### Build Addon

```bash
investwise build
```

### Package for Distribution

```bash
investwise package
```

### Test Setup

```bash
investwise test
```

## Development Server

The development server provides:

- Hot reload functionality
- File watching
- Auto-building
- Health check endpoints

### API Endpoints

- `GET /health` - Health check
- `GET /status` - Addon status and last modified time
- `GET /manifest.json` - Addon manifest
- `GET /addon.js` - Built addon code
- `GET /files` - List of built files
- `GET /test` - Test connectivity

## Usage in Addon Projects

Add to your addon's `package.json`:

```json
{
  "scripts": {
    "dev:server": "investwise dev"
  },
  "devDependencies": {
    "@investwise/addon-dev-tools": "^1.0.0"
  }
}
```

## Architecture

This package is separate from `@investwise/addon-sdk` to:

- Keep the SDK lightweight for production
- Avoid unnecessary dependencies in addon bundles
- Provide optional development tooling

## License

MIT
