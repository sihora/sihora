# CLAUDE.md — Sihora

Cross-platform declarative UI framework in pure Rust. SwiftUI/Compose-style API, own wgpu renderer, own design system, SwiftData-like data layer. No HTML/CSS.

## Read first
- `docs/PRD.md` — what & why
- `docs/ARCHITECTURE.md` — layers, pipeline, crate rules
- `docs/ADR.md` — decisions (do not contradict; propose new ADR instead)
- `docs/ROADMAP.md` — current milestone + exit criteria
- `docs/DESIGN_SYSTEM.md` — tokens/components

## Current milestone
M11 — v0.1 public release. (Update this line when moving on.)

## Workspace
- Monorepo, Cargo workspace, two published crates: `sihora` + `sihora-data-derive`.
- `sihora` is the single user-facing crate with feature flags (`full`, `render`, `text`, `widgets`, `primitives`, `platform`, `data`).
- `sihora-data-derive` is the proc macro crate for `#[derive(Model)]`.
- Examples under `examples/`, depend only on `sihora`.

## Hard rules
- Only the `platform` module touches winit/web-sys/OS APIs.
- `sihora` core+layout modules must compile on `wasm32-unknown-unknown`: no threads, no blocking I/O, no `std::fs`.
- Widgets use semantic theme tokens only. No raw colors/sizes.
- No `Rc<RefCell<T>>` for user-facing state; use signal/handle IDs.
- No `unwrap()`/`expect()` in library code except proven invariants (comment why).
- `unsafe` only with `// SAFETY:` comment.
- Public API: doc comments + example.
- Stay within current milestone scope. Don't build ahead.

## Commands
```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p sihora --target wasm32-unknown-unknown --no-default-features --features "render,text,widgets,primitives,data"
cargo run -p <example>
```

## Workflow
1. Read relevant docs + current milestone.
2. Plan briefly; keep changes small.
3. Implement + tests.
4. Run fmt, clippy, test, wasm check.
5. If a decision changes architecture → append ADR in `docs/ADR.md`.

## Git
- Tidak boleh ada `Co-Authored-By: Claude` atau mention "Generated with Claude Code" di commit message / PR body.

## Style
- Edition 2021+, MSRV = stable − 2.
- Small modules, explicit names, no premature abstraction.
- Prefer generics inside components, `AnyView` at component boundaries.
