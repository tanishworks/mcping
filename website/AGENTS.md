# AGENTS.md — website/

Instructions for autonomous agents (and impatient humans) working on the
mcping site. Read this before changing anything.

## What this is

- Vite + React 19 + TypeScript SPA, two pages: `/` (home) and `/docs`.
- Lives in `website/` inside the mcping repo; deployed to GitHub Pages at
  https://tanishworks.github.io/mcping/ by
  [`.github/workflows/website.yml`](../.github/workflows/website.yml).
- Vite `base` is `/mcping/`; do not change it unless the repo moves.
- The build copies `dist/index.html` → `dist/404.html` for SPA deep links.

## Rules

### Colors (hard rule)

Every color is a token in [`src/theme/theme.tsx`](src/theme/theme.tsx)
(`colors` object). Components and CSS may **only** reference theme colors:

- In stylesheets/inline styles: `var(--bg)`, `var(--fg)`, `var(--muted)`,
  `var(--hairline)`, `var(--gradient-a)` … (names are the tokens kebab-cased).
- From JS/canvas: `useTheme()` → `color('fg')` (resolved value) or
  `varName('fg')` (CSS var string).

Never hard-code a color — not even one used once, not even gray. If it is a
new color, add a token to `colors` in `theme.tsx` first (both `light` and
`dark` values; the scheme is pure black/white monochrome plus grays).
Fallback definitions also exist in `src/theme/theme.css` and must mirror
`theme.tsx`. The active scheme comes from `prefers-color-scheme`
(`document.documentElement.dataset.theme`), don't add a manual toggle unless
asked.

### Typography & spacing

Font stacks are CSS variables in `src/theme/theme.css`:
`--font-body` (Inter), `--font-mono` (JetBrains Mono), `--font-pixel`
(Press Start 2P). Hierarchy: pixel = display/hero/labels, mono = nav &
terminals & code, Inter = prose. All fonts are **local woff2** files in
`src/assets/fonts/` declared in `src/fonts.css` with `font-display: block`.
Do not add remote font imports. If a new weight/style is needed, download the
woff2 and add an `@font-face`, don't link a CDN.

Spacing/type scale (hard rules, mirrors the tagg site):

- Root size: `html { font-size: clamp(16px, 14.5px + 0.25vw, 18px) }`; sizes
  everything below in rem.
- Line-heights: 1.6 UI/labels, 1.7 body copy/cards, 1.8 prose — never mix
  other values; keep heading line-heights 1.15–1.5.
- Spacing in rem on the `.25/.5/.9/1.25/1.5/2/2.5rem` cuts; sections use
  `padding: 2.5rem 1.5rem` and the standard container is `max-width: 72rem`.
- Display sizes use conventional clamps (e.g.
  `clamp(2.2rem, 8vw, 4.6rem)` for the hero).
- Card grids are `repeat(auto-fit, minmax(15.5rem, 1fr))` and the grid IS the
  `AsciiReveal` wrapper (`className="cards"`), not a section on top of it —
  otherwise the reveal wrapper becomes the only grid child and the cards
  stack vertically.

### ascii-reveal

Reveal effects (`AsciiReveal`, `AsciiSkeleton`) from the `ascii-reveal` npm
package load hero/cards/docs. Layout rules:

- Reveals must start only after fonts are ready (`useFontsReady()` /
  `document.fonts.ready`) so text never swaps typeface mid-animation.
- Animate wrappers, not data-critical islands: mark toolbars/interactive
  chrome with `data-ascii="skip"` (the pill nav already has it).
- Cards get `data-ascii="surface"`, buttons/pills `data-ascii="fade"`.
- ascii-reveal reads its colors from `--ascii-*` vars (already mapped to
  theme tokens in `theme.css`; keep it that way).

### Header behavior

`SiteHeader` hides on scroll-down and reappears on scroll-up or pointer-at
top edge; the pill nav is centered with an animated sliding indicator measured
from real DOM geometry (re-measure after `useFontsReady()` / resize — keep
that wiring when touching it).

### Docs page `/docs`

- Content source: `src/content/docs.md` (imported with Vite's `?raw` and
  rendered by `components/Markdown.tsx`).
- Sidebar outline is derived from `##`/`###` headings via slugged ids — keep
  `slugify` in `hooks/useScrollSpy.ts` as the single id generator used by
  both the markdown renderer and the outline parser.
- Sidebar: permanent left column ≥ 960px, drawer + scrim below that; the
  menu button (left of the pill nav) toggles it; active section is bold.

## Nice to keep

- `npm run build` must pass (it runs `tsc -b`) — also run `npm run lint`.
- No new runtime dependencies without a reason; ascii-reveal, react-markdown,
  react-router-dom are already there.
- Conventional commits (`feat(website): …`, `fix(website): …`) on a feature
  branch; the root [README](../README.md) points back here.
