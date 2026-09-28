# Contributing to kdn

Thanks for helping out. This is a small Rust CLI, so the process is short.

## Prerequisites

- Linux or macOS
- Stable Rust toolchain (`rustup update stable`)
- A running Kaiden desktop app with the `api.server.enabled` preference on, to try commands for real

## Workflow

1. Fork and branch from `main`.
2. Make your change. Keep the diff focused; open a separate PR for unrelated fixes.
3. Run the same checks as CI before pushing:

   ```sh
   make ci-checks   # cargo fmt --check, cargo clippy --all-targets -D warnings, cargo test
   ```

   Clippy runs with the `pedantic` group and `unsafe_code` is forbidden (see `[lints]` in `Cargo.toml`).
4. Open a pull request and fill in the template. CI also runs `cargo deny check` (advisories, licenses, sources) on every PR.

## Conventions

- Every new source file starts with the `Copyright (C) 2026 Red Hat, Inc.` / `SPDX-License-Identifier: Apache-2.0` header. Copy it from an existing file.
- Unit tests live inline in `#[cfg(test)] mod tests` next to the code.
- Commit messages use a short type prefix, for example `feat:`, `fix:`, `chore:`, `docs:`.
- `AGENTS.md` describes the code layout, how to add a subcommand, and the release procedure. Update it and `README.md` when behaviour changes.

## Releases

Maintainers cut releases by tagging `vX.Y.Z`; cargo-dist builds and publishes the binaries. The exact steps are in `AGENTS.md`.

By contributing you agree that your contributions are licensed under the Apache License 2.0 (see `LICENSE`).
