# Addon Quick Start

## Create an Addon

```bash
npx @investwise/addon-dev-tools create my-addon
cd my-addon
npm install
npm run dev
```

## Structure

```
my-addon/
├── manifest.json
├── package.json
├── src/
│   └── index.tsx
└── vite.config.ts
```

## Development

Run the addon development server and enable addon dev mode in the main app.
