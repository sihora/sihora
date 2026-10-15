# Design System Spec — Sihora DS (working name)

Status: Draft. Fill `TBD` before M7.

## 1. Principles (pick 3–5, TBD)
- e.g. Calm · Clear hierarchy · Fast feedback · Soft geometry · Motion with purpose

## 2. Architecture
- **Primitives** (`sihora-primitives`): unstyled. Always available.
- **Tokens** (`sihora-theme`): semantic values, light + dark.
- **Components** (`sihora-widgets`): styled, read tokens only.
- Opt-out: `sihora = { default-features = false }` → primitives only.
- Per-widget override via style traits (`ButtonStyle`, `FieldStyle`, …).

## 3. Tokens

### 3.1 Color
- Base palettes generated in OKLCH, 11 steps (50…950): `neutral`, `brand`, `success`, `warning`, `danger`, `info`.
- Semantic roles (widgets use only these):
  - `bg`, `bg.subtle`
  - `surface`, `surface.raised`, `surface.overlay`
  - `text.primary`, `text.secondary`, `text.disabled`, `text.on_accent`
  - `border`, `border.strong`, `focus.ring`
  - `accent`, `accent.hover`, `accent.pressed`
  - `success`, `warning`, `danger` (+ `.subtle`, `.text`)
- Light + dark required. Contrast WCAG AA verified by test.

### 3.2 Typography
- Family: TBD (Inter or Geist, OFL). Mono: TBD.
- Scale: `caption 12`, `body.sm 13`, `body 15`, `label 14`, `title.sm 17`, `title 20`, `headline 24`, `display 32`.
- Weights: 400, 500, 600.

### 3.3 Spacing (4px grid)
`0, 2, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64`

### 3.4 Radius
`none 0`, `sm 4`, `md 8`, `lg 12`, `full 9999`

### 3.5 Elevation
- Strategy TBD: shadow-based or border-based. Levels: `0, 1, 2, 3`.

### 3.6 Motion
- Durations: `fast 120ms`, `normal 200ms`, `slow 320ms`.
- Easing: `standard`, `emphasized`; default spring (stiffness/damping TBD).
- Respect OS reduce-motion.

### 3.7 Density
- `compact`, `regular` (default), `comfortable` → scales spacing + control heights.

## 4. Components (v0.1 scope, ≤10)
| # | Component | Variants | Required states |
|---|---|---|---|
| 1 | Text | type scale roles | — |
| 2 | Icon | sizes | — |
| 3 | Button | primary, secondary, ghost, destructive · sm/md/lg | hover, pressed, focus, disabled, loading |
| 4 | TextField | default, error | focus, disabled, error, placeholder |
| 5 | Toggle | — | on/off, focus, disabled |
| 6 | Checkbox | — | checked, indeterminate, focus, disabled |
| 7 | Card | flat, raised | — |
| 8 | ListRow | with leading/trailing | hover, selected, pressed |
| 9 | Dialog | — | open/close transition |
| 10 | Divider | h/v | — |

Next: Menu, Tooltip, Tabs, Sidebar, Sheet, Slider, Select, Toast.

## 5. Component checklist (every component)
- [ ] Uses semantic tokens only (no raw hex/px)
- [ ] All states implemented
- [ ] Visible focus ring, keyboard operable
- [ ] Light + dark
- [ ] Density aware
- [ ] A11y role + label
- [ ] Appears in gallery app with every variant/state
- [ ] Snapshot test

## 6. Platform adaptation (behavior, not look)
- Shortcuts: Cmd (macOS) vs Ctrl.
- Scroll physics per platform.
- Back gesture (mobile), safe areas.
- Visual identity stays the same across platforms.

## 7. Tooling
- Source of truth: Figma + Tokens Studio → `tokens.json` → `build.rs` generates `Theme`.
- Gallery app: `examples/gallery`.

## 8. Versioning
- `Theme::v1()`. Breaking visual changes → `v2`, opt-in. `Theme::default()` pinned per framework major.
