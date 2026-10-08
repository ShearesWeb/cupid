# Helios Design System (HDS) — Brand & Design Guide

An agent-ready recreation of **Helios**, HashiCorp's open-source design system (commonly "HDS"). Helios provides the building blocks — tokens, foundations, and components — used across HashiCorp's customer-facing products (HCP Terraform, Vault, Consul, Nomad, Boundary, Packer, Waypoint).

This project packages the real Helios **design tokens**, the **Flight Icons** set, product **logos**, faithful **component primitives**, and an interactive **HCP Terraform UI kit** so design agents can produce on-brand HashiCorp interfaces in both **light and dark** mode.

## Sources
- **Design system site:** https://helios.hashicorp.design/
- **Monorepo:** https://github.com/hashicorp/design-system (MPL-2.0)
  - Tokens lifted verbatim from `packages/tokens/dist/products/css/tokens.css`
  - Icons imported from `packages/flight-icons/svg/` (the Flight Icons set)
- **Tokens package:** `@hashicorp/design-system-tokens` (npm)

> The reader is not assumed to have access to the above; values are captured here in `tokens/`.

---

## Brand context
HashiCorp builds infrastructure-automation tooling. Helios exists to give every product a **single, consistent, accessible** design language. The system meets or exceeds **WCAG 2.2 AA** out of the box, and its semantic color pairings are accessible by construction. Helios is framework-aware (originally Ember, now React-friendly) but its *foundations* are plain CSS custom properties — which is exactly what this project ships.

The brand reads as **engineering-grade, calm, and precise**: neutral greys, a confident blue for action, restrained status colors, system typography, hairline borders, and soft layered shadows. Each product owns one accent color (Terraform purple, Vault yellow, Consul pink, etc.) but the application chrome stays neutral.

---

## CONTENT FUNDAMENTALS
How HashiCorp / Helios writes UI copy:

- **Voice:** clear, concise, confident, and human. Favor plain language over jargon; explain, don't impress.
- **Person:** address the user as **"you"**; refer to the product/system as itself ("Terraform will plan…"), not "we".
- **Casing:** **Sentence case** everywhere — buttons, headings, labels, menu items ("New workspace", not "New Workspace" or "NEW WORKSPACE"). Reserve all-caps only for tiny eyebrow labels / table headers (tracked, letter-spaced).
- **Buttons:** lead with a **verb** describing the outcome — "Create", "Confirm & apply", "Sign in", "Continue with SSO". Avoid "OK"/"Submit".
- **Tone in errors:** factual and non-blaming. State what happened and what to do: "Authentication to the AWS provider was rejected." Not "Oops!".
- **Numbers & status:** terse and scannable — "3 added, 0 changed, 0 destroyed", "142 resources", "2h ago".
- **Punctuation:** minimal. No exclamation marks in product UI. Use the ampersand in compact action labels ("Confirm & apply").
- **Emoji:** **never** in product UI.
- **Terminology:** consistent product nouns — *workspace, run, plan, apply, state, variable, module, registry, drift*.

---

## VISUAL FOUNDATIONS
Answers to the "how does this brand look" questions:

- **Color vibe:** cool, neutral, restrained. A near-black/grey text ramp (`neutral-700→0`), a single **action blue** (`#1060ff`), and four status hues (green/amber/red, plus purple for "highlight"). Color is used *semantically*, never decoratively. Backgrounds are flat — **no gradients** in product chrome (product *brand* gradients exist only for marketing/illustration).
- **Backgrounds:** solid surfaces. Page = white (`#ffffff`) / faint (`#fafafa`); cards sit on white. No textures, no patterns, no hero imagery in-app. Dark mode uses a near-black page (`#0a0a0b`) with a slightly lifted surface (`#161719`).
- **Typography:** the **native system font stack** (`-apple-system, BlinkMacSystemFont, "Segoe UI"…`) for both Display (headings/UI) and Body (text); `ui-monospace, Menlo, Consolas` for Code. Three scales — Display, Body, Code — each in three to five steps. Weights 400/500/600/700. Display ≥300 uses a slight negative tracking (`-0.5px`).
- **Spacing:** a **4px grid** (4, 8, 12, 16, 24, 32, 48, 64…). Generous but not airy; dense tables, comfortable forms.
- **Corner radii:** small and consistent — `3px` (x-small, e.g. checkbox/toggle), `5px` (small, controls/buttons), `6px` (medium, cards/inputs grouping), `8px` (large, modals/login). Pills only for count badges.
- **Borders:** **hairlines.** Borders are translucent neutral (`#656a76` at 10–40% alpha) so they read on any surface. The signature card treatment is the **"surface" shadow** — a 1px hairline *baked into* the box-shadow (`0 0 0 1px …` + soft drop) rather than a separate border.
- **Shadows / elevation:** two systems. **Elevation** = shadow only (for already-bordered elements); **Surface** = hairline + shadow (base → low → mid → high → higher → overlay). Soft, low-spread, cool-grey. In dark mode shadows deepen to black alpha and the hairline switches to white alpha.
- **Focus:** a highly visible **2-ring focus**: inset 1px action-blue + 3px outer halo (`#5990ff`). Critical controls use a red variant. Never removed — accessibility-critical.
- **Hover states:** surfaces step *up* one tone (`surface-interactive` → `…-hover` → `…-active`); the action blue *darkens* on hover (`#1060ff`→`#0c56e9`→`#0046d1`). Text links darken similarly.
- **Press states:** color shift to the `-active` tone (no scale/shrink transforms).
- **Transparency & blur:** used sparingly — translucent borders and the tooltip/overlay scrims. No glassmorphism.
- **Motion:** quick and functional. `0.15s` for color/hover, `0.2s` for control transitions. Signature easings: toggle thumb `cubic-bezier(0.68,-0.2,0.265,1.15)` (slight overshoot), tab indicator `cubic-bezier(0.5,1,0.89,1)` (sliding underline), tooltip pop `cubic-bezier(0.54,1.5,0.38,1.11)`. **No** decorative/looping animation, no parallax.
- **Imagery:** product chrome is imagery-free. Where brand imagery appears (marketing), it is clean and geometric, built on the product accent colors and the hexagon motif.
- **Cards:** white surface, `6px` radius, the **surface-base** hairline-shadow, `16–24px` padding. Interactive cards raise to `surface-mid` on hover.

---

## ICONOGRAPHY
- **System:** **Flight Icons** — HashiCorp's own open-source icon set (`@hashicorp/flight-icons`). Imported as standalone SVGs into `assets/icons/`.
- **Format:** single-path (or small multi-path) **SVG**, `16×16` and `24×24` artboards, `fill="currentColor"` — so an icon inherits text color.
- **Tinting:** because `currentColor` does **not** flow through an `<img>` tag, monochrome icons are rendered through the **`Icon`** component, which uses a CSS `mask` to paint the SVG in `currentColor` (works on colored buttons and in dark mode). Use `<Icon src="…/name-16.svg" />`, not a bare `<img>`, anywhere the icon must match the text color.
- **Product & vendor marks:** multicolor logos (HashiCorp, Terraform, Vault, AWS, etc.) keep their own colors and are used as `<img>` — see `assets/brand/`.
- **Sizing:** 16px is the workhorse size in dense UI; 20–24px for emphasis. Don't recolor multicolor marks.
- **Emoji / unicode as icons:** **never.** Always a Flight Icon.
- **Substitutions:** none. All icons here are the genuine Flight Icons set (a curated ~50-icon subset; the full set has 600+ glyphs at https://helios.hashicorp.design/icons/library).

---

## Index / manifest

**Foundations (global CSS)** — link `styles.css`, which `@import`s:
- `tokens/colors.css` — core palette, product brand colors, semantic light + `[data-theme="dark"]`
- `tokens/typography.css` — font stacks, weights, Display/Body/Code scales
- `tokens/spacing.css` — 4px spacing scale, radii, sizing, motion
- `tokens/elevation.css` — elevation & surface shadows, focus rings (light + dark)

**Components** (`window.HeliosDesignSystem_f36c58.*`) under `components/`:
- `actions/` — **Button**, **IconButton**
- `forms/` — **TextInput**, **Select**, **Checkbox** (+ radio), **Toggle**
- `feedback/` — **Badge**, **Alert**
- `containers/` — **Card**
- `navigation/` — **Tabs**
- `media/` — **Icon**, **Avatar**

**UI kits** under `ui_kits/`:
- `hcp/` — interactive HCP Terraform application (sign-in → workspaces → run history), light/dark

**Foundation cards** under `foundations/` — specimen tiles shown in the Design System tab (Colors, Type, Spacing, Brand).

**Assets** under `assets/`:
- `icons/` — Flight Icons (16px, currentColor SVG)
- `brand/` — HashiCorp + product logos (color SVG)

**Other:** `SKILL.md` (Agent-Skill entry point), per-component `*.prompt.md` usage notes.

---

## Caveats / substitutions
- **Dark mode** is an interpretation. Helios products historically ship **light-only** for product surfaces; the `[data-theme="dark"]` semantic scope here is an on-brand, minimalistic dark palette designed to pair with the real light tokens — it is **not** an official HashiCorp dark theme.
- **Fonts:** Helios products use the native **system font stack** (no webfont), so there is nothing to download or substitute.
- **Spacing scale** tokens (`--hds-spacing-*`) are modeled on HDS layout usage; HDS exposes spacing largely through layout components rather than a published numeric token set.
- Demo data in the UI kit is fictional.
