# Architecture Decision Records — Sihora

Format: Context · Decision · Consequences. Status: Accepted unless noted.
New decisions: append, never rewrite. Superseded → mark and link.

---

## ADR-001: Language = Rust
- **Context:** Need performance, memory safety, single language across all targets incl. wasm.
- **Decision:** Pure Rust. No JS/Dart/Kotlin in user-facing API.
- **Consequences:** + Safety, perf, wasm support. − Ownership/closure DX friction; longer compile times.

## ADR-002: Own GPU renderer, no native widgets
- **Context:** Options: native widgets (RN), webview (Tauri), own renderer (Flutter).
- **Decision:** Own renderer on wgpu.
- **Consequences:** + Pixel-identical across platforms, full control. − Must build text, a11y, IME, scrolling physics ourselves.

## ADR-003: Retained mode + fine-grained signals
- **Context:** Immediate mode (egui) vs retained + VDOM diff vs retained + signals.
- **Decision:** Retained tree, fine-grained signals (Leptos/Solid style). No VDOM diff.
- **Consequences:** + Minimal updates, good a11y/perf. − Reactive runtime complexity; ownership scopes needed.

## ADR-004: Declarative modifier API (SwiftUI/Compose style), no HTML/CSS
- **Context:** Target DX is SwiftUI/Compose.
- **Decision:** Components = functions returning `impl View`; styling via modifiers and theme tokens.
- **Consequences:** + Familiar to mobile devs, type-safe. − No CSS ecosystem reuse.

## ADR-005: Builder API first, macro DSL later
- **Decision:** Ship builder/method-chain API first. `view!{}` macro optional, post-v0.1.
- **Consequences:** + Debuggable, IDE-friendly. − Slightly more verbose early.

## ADR-006: Cargo workspace monorepo
- **Context:** Repo-per-feature vs monorepo.
- **Decision:** Single repo, Cargo workspace, crates split by layer. Facade crate `sihora`.
- **Consequences:** + Atomic cross-crate changes, single CI. − Must enforce crate boundaries manually. Split repos only when a crate is stable and has outside consumers.

## ADR-007: Desktop first, then web, then mobile
- **Decision:** M1–M6 desktop. Web next. Mobile last.
- **Consequences:** + Fast iteration. − Risk of desktop-only assumptions → mitigated by wasm32 compile check in CI from day 1; no blocking/threads/`std::fs` in `core`.

## ADR-008: Web = canvas + hidden DOM overlay
- **Decision:** Render to canvas (WebGPU, WebGL2 fallback). Hidden DOM for text input, IME, clipboard, a11y.
- **Consequences:** + Same renderer everywhere. − Bigger bundle, slower first paint, not for SEO sites.

## ADR-009: Reuse low-level foundations
- **Decision:** Use winit (windowing), wgpu (GPU), cosmic-text or parley (text), accesskit (a11y), raw-window-handle. Build everything above.
- **Consequences:** + Years saved. − Dependent on upstream release cadence.

## ADR-010: Data layer uses ID handles, not Rc<RefCell<T>>
- **Context:** SwiftData uses classes (shared mutable refs). Rust has no classes.
- **Decision:** Store owns data in arenas; users hold `Handle<T>` (Copy, ID-based). Fields exposed as signals. Behavior via `impl`/traits.
- **Consequences:** + No borrow panics, no cycles, serializable. − Access via store indirection.

## ADR-011: Own design system, layered and replaceable
- **Context:** Options: native-per-OS, Material 3, own.
- **Decision:** Own design system as default. Layers: primitives (headless) → tokens → styled widgets. Opt-out via `default-features = false`.
- **Consequences:** + Identity, control. − Design workload on solo dev; must keep token discipline.

## ADR-012: Theme versioning, opt-in upgrades
- **Decision:** `Theme::v1()`, `Theme::v2()`; `Theme::default()` pinned per major version.
- **Consequences:** Apps don't change look unexpectedly on update.

## ADR-013: Architecture-agnostic, MVVM as documented default
- **Decision:** Framework doesn't enforce a pattern. Docs/examples use MVVM (ViewModel = struct of signals). MV and Elm/TCA-style supported; optional crate later. MVP not supported.
- **Consequences:** + Flexibility. − Need clear guidance docs.

## ADR-014: Entry point = App trait + run()
- **Decision:** `impl App for MyApp { fn body(&self) -> impl Scene }` + `sihora::run(MyApp)`. `#[sihora::main]` later for platform glue.

## ADR-015: Type-erase at component boundaries
- **Context:** Deep generic view types blow up compile times.
- **Decision:** Generics within a component; `AnyView` at `#[component]` boundaries.

## ADR-016: Text shaping via cosmic-text
- **Context:** M2 (Text) requires font loading, shaping, glyph rasterization, line wrapping, and font fallback for Latin/CJK/emoji. Two candidates: cosmic-text (used by COSMIC desktop, Lapce, iced) and parley (new Linebender crate, successor to piet-text).
- **Decision:** cosmic-text. It is battle-tested in production desktop apps, has mature CJK/emoji/font-fallback support via built-in fontdb + swash, and a well-documented API. parley is newer and evolving fast but not yet at parity for our needs.
- **Consequences:** + Working text rendering quickly. + Built-in system font discovery. − Larger transitive dependency surface. Can revisit parley for M9 (web) if wasm story improves.

## ADR-017: Web platform via conditional compilation + Canvas 2D text
- **Context:** M9 requires wasm32 support. Key blockers: (1) wgpu/winit GPU init is async on web, (2) AccessKit has no web backend, (3) cosmic-text on wasm has no system fonts, (4) rusqlite doesn't compile on wasm.
- **Decision:** `#[cfg(target_arch = "wasm32")]` selects web modules in `sihora-platform` and `sihora-data`. Web text rasterization uses an offscreen Canvas 2D element (browser provides fonts). Data layer uses in-memory HashMap backend on web. GPU init via `wasm_bindgen_futures::spawn_local`. AccessKit skipped on web (a11y via hidden DOM later).
- **Consequences:** + Same app code compiles for desktop and web without changes. + No font bundling needed. − Canvas 2D text is per-string (not per-glyph), uses more atlas space. − Web data is in-memory only (no persistence across reloads yet; IndexedDB/OPFS can be added).

## ADR-018: Mobile platform via separate module + touch translation
- **Context:** M10 requires Android and iOS support. winit 0.30 supports both platforms. AccessKit has no Android backend. On mobile, GPU surfaces must be released on suspend/resume lifecycle.
- **Decision:** Separate `mobile.rs` module in `sihora-platform`, selected via `#[cfg(any(target_os = "android", target_os = "ios"))]`. Touch events translated to existing PointerPressed/Released/Moved events in shared `common.rs`. No AccessKit on mobile (platform a11y later). Proper suspend/resume drops and recreates GPU state. Android entry point via thread-local `AndroidApp` + `set_android_app()`. iOS uses standard `EventLoop::new()`.
- **Consequences:** + Same rendering pipeline and app code on all platforms. + Touch works without new event types. − No accessibility on mobile yet. − Android builds need NDK + cargo-ndk. − No multi-touch/gestures yet (single-finger only).

---

## Pending decisions
- ~~ADR-0xx: cosmic-text vs parley~~ → ADR-016
- ADR-0xx: SQLite crate (rusqlite vs sqlx vs pure-Rust redb + own query)
- ~~ADR-0xx: Mobile shell (winit backends vs custom)~~ → ADR-018
- ADR-0xx: License (MIT OR Apache-2.0 proposed)
- ADR-0xx: Final name
