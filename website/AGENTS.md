# AGENTS.md — website/

Instructions for autonomous agents (and impatient humans) working on the
mcping site. Read this before changing anything.

## What this is

- Vite + React 19 + TypeScript SPA with `/` (home), `/about`, and `/docs`.
- Lives in `website/` inside the mcping repo; deployed to GitHub Pages at
  https://tanishworks.github.io/mcping/ from the `gh-pages` branch (CI pushes
  `website/dist` there via
  [`.github/workflows/website.yml`](../.github/workflows/website.yml);
  `npm run deploy` does the same locally with the `gh-pages` package).
- Vite `base` is `/mcping/`; do not change it unless the repo moves.
  `BrowserRouter` must keep its matching `basename={import.meta.env.BASE_URL}`
  — without it, reloading a deep link like `/mcping/docs` no longer matches
  any route and bounces to the site root.
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

### Icons

Favicons and the hero icon come from the repo's `icon/exportss/png/` exports —
`website/public/favicon-32.png` (favicon, dark appearance),
`website/public/apple-touch-icon.png`
(home-screen icon) and `website/src/assets/icon.png` (landing hero) are all
`sips`-downscaled from `icon/exportss/png/Icon-iOS-Dark-512@1x.png`; regenerate
with `sips -z <size> <size> <source> --out <dest>` rather than hand-editing.
The light-appearance favicon `website/public/favicon-32-light.png` is derived
from the same monochrome dark export: run
`node scripts/invert-png.mjs` (inverts RGB, keeps alpha) on
`Icon-iOS-Dark-512@1x.png`, then `sips`-downscale the result to 32×32. The
icon project has no light export, so don't regenerate it from any other
source. `index.html` switches favicons per scheme with `media` attributes
(light icon in `prefers-color-scheme: light`, dark icon in dark), mirroring
the `theme-color` meta tags; bump the `?v=` cache-buster when changing icon
files.
The hero `<img>` must keep `data-ascii="fade"` and its intrinsic
width/height attributes (no CLS).

### ascii-reveal

Reveal effects (`AsciiReveal`, `AsciiSkeleton`) from the `ascii-reveal` npm
package load the home hero, the About/Explore page and the docs page. Layout
rules:

- Reveals must start only after fonts are ready (`useFontsReady()` /
  `document.fonts.ready`) so text never swaps typeface mid-animation.
- Animate wrappers, not data-critical islands: mark toolbars/interactive
  chrome with `data-ascii="skip"` (the pill nav already has it).
- Cards get `data-ascii="surface"`, buttons/pills `data-ascii="fade"`.
- ascii-reveal reads its colors from `--ascii-*` vars (already mapped to
  theme tokens in `theme.css`; keep it that way).

### Header behavior

The bar is **always visible** — there is no hide-on-scroll behavior, by
design (keep it simple). `SiteHeader` renders a fixed, click-transparent
`.site-headroom` wrapper; only its children take pointer events. The pill nav
is centered with an animated sliding indicator measured from real DOM
geometry (re-measure after `useFontsReady()` / resize — keep that wiring when
touching it).

Below **960px** (the same breakpoint where the docs sidebar becomes a
drawer) the whole bar docks to the **bottom edge** for thumb navigation and
the tabs become icon-over-label (`lucide-react` icons, 1.6rem, 0.72rem mono
labels); desktop tabs stay text-only. `body` gets bottom padding for
clearance, except on the home page, whose viewport-height hero reserves its
own space. `viewport-fit=cover` + `env(safe-area-inset-bottom)` keep the bar
clear of the home indicator. The pill itself must keep only its theme
background — no extra surfaces/shadows; the repo icon links live in their own
solid pill at the right edge.

### Home and About pages

The home page is a centered, viewport-height landing hero only: keep the app
icon, title, summary, install command, and adjacent Releases/About actions.
Do not add content beneath the actions. About/Explore is a concise, copy-ready
quick-use page for people who already installed the binaries: show a handful
of tasks, each with a pasteable example command and a copy button; do not
repeat installation instructions or long feature explanations. The command
examples and behavior in `src/content/docs.md` remain canonical; keep the
About examples in sync with its “Common tasks” section.

### Icons (lucide)

UI icons are **lucide-react** for interface glyphs: `House` / `BookOpen` for
the tabs, `Menu` for the docs drawer toggle, `CircleDot` / `Tag` for issues
and releases, and `Copy` / `Check` for the install pill. The repository link
uses the official GitHub mark from Simple Icons via `components/GitHubMark.tsx`
(lucide does not include brand marks); it uses `currentColor` to follow the
theme. Pass
`size: 16, strokeWidth: 1.6` (shared `iconProps` in `SiteHeader`) to match
the hairline weight; sizes come from CSS where a component scales them.
Do not hand-draw new SVG icons — use Lucide for UI icons and sourced marks
for brand logos.

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
