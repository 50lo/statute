# Releasing

Releases are cut locally with [cargo-release](https://github.com/crate-ci/cargo-release) and built on CI by [cargo-dist](https://github.com/axodotdev/cargo-dist).

## Before you start

- Install cargo-release once: `cargo install cargo-release --locked`.
- Be on an up-to-date `main` with a clean working tree.
- Make sure `## Unreleased` in `CHANGELOG.md` lists every user-visible change. It becomes the release notes.

## Cut a release

Pick the part of the version to increase: `patch` for fixes, `minor` for new features, `major` for breaking changes.

```sh
cargo release minor            # dry run: shows what would change
cargo release minor --execute
```

This:

1. Bumps the version in `Cargo.toml`, `Cargo.lock`, `plugin/.claude-plugin/plugin.json` and `plugin/.codex-plugin/plugin.json`.
2. Renames `## Unreleased` in `CHANGELOG.md` to `## X.Y.Z - YYYY-MM-DD` and adds an empty `## Unreleased` above it.
3. Commits `Release vX.Y.Z`, tags `vX.Y.Z` and pushes both to `main`.

The tag push starts the Release workflow in GitHub Actions. It builds `statute` for macOS and Linux on x86_64 and arm64, plus a shell installer and checksums, and publishes a GitHub release whose notes are the `X.Y.Z` section of `CHANGELOG.md`.

## Publish to crates.io

cargo-release does not publish to crates.io. After the release, publish from the tagged commit:

```sh
cargo publish
```

## Change the release setup

Targets, installers and other build settings are in `dist-workspace.toml`. After changing them, or upgrading cargo-dist, run `dist generate` to regenerate `.github/workflows/release.yml`. Do not edit that workflow by hand. The cargo-release settings are in `[package.metadata.release]` in `Cargo.toml`.
