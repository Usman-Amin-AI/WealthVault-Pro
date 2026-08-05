.PHONY: dev build test clean

dev:
pnpm dev

build:
pnpm build

test:
pnpm test

clean:
rm -rf node_modules dist target

install:
pnpm install

lint:
pnpm lint

format:
pnpm format
