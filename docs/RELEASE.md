# Release Process

## Versioning

Follow semantic versioning (semver).

## Steps

1. Update CHANGELOG.md
2. Bump version in package.json
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions builds releases

## Platforms

- Windows (NSIS installer)
- macOS (DMG)
- Linux (AppImage, deb)
- Docker image
