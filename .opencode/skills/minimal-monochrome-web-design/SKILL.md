---
name: minimal-monochrome-web-design
description: Apply the mcping website's restrained monochrome visual language to another website while adapting to that project's stack, content, and existing constraints.
---

# Minimal monochrome web design

Use this skill when designing or refining a website that should share the visual character of the mcping site: quiet, typography-led, monochrome, responsive, and precise. Treat this as a design system to adapt, not a request to copy mcping's product, pages, or implementation.

## Boundaries first

- Inspect the target project's instructions, existing design system, routes, components, and build setup before making changes. Follow its conventions and preserve unrelated work.
- Keep the target site's framework, routing, content, branding, and behavior. Do not import mcping-specific names, copy, app icon, routes, deployment assumptions, React packages, or ASCII-reveal effects unless explicitly requested and appropriate.
- Prefer extending existing tokens and primitives over introducing parallel systems. Avoid new dependencies unless necessary.
- Keep this visual direction recognizable but subordinate to the target project's accessibility, product needs, and explicit requirements. Ask before replacing established branding or changing behavior beyond the requested scope.

## Visual principles

- Use generous whitespace, strong alignment, restrained borders, and a small number of purposeful surfaces.
- Establish hierarchy through scale, weight, spacing, and contrast rather than colorful decoration, gradients, or shadows.
- Favor rounded pills for compact navigation/actions and modestly rounded rectangles for content cards or code surfaces. Keep surfaces flat and borders hairline-thin.
- Use animation sparingly for meaningful state changes. Respect reduced-motion preferences; never make motion necessary to understand content or operate controls.
- Make content the focal point. Avoid adding sections, ornament, or marketing copy simply to fill space.

## Color system (required convention)

Use a semantic token system as the single source of truth for every color. Never sprinkle literal colors through components, stylesheets, or inline styles. If a color is needed, define a named token first and consume that token everywhere.

The reference palette is pure black and white with neutral grays:

| Token role | Light | Dark | Use |
| --- | --- | --- | --- |
| `bg` | `#ffffff` | `#000000` | Page and solid surface background |
| `fg` | `#000000` | `#ffffff` | Primary text, emphasis, active indicator |
| `muted` | `#5e5e5e` | `#a1a1a1` | Secondary text |
| `faint` | `#9c9c9c` | `#575757` | Tertiary labels and subtle details |
| `hairline` | `rgba(0,0,0,.14)` | `rgba(255,255,255,.16)` | Dividers and outlines |
| `hover` | `rgba(0,0,0,.05)` | `rgba(255,255,255,.07)` | Subtle hover/inline-code fill |
| `gradient-a/b/c` | `#e7e7e7` / `#f2f2f2` / `#d6d6d6` | `#161616` / `#212121` / `#2b2b2b` | Optional very restrained ambient background |
| `shadow` | `rgba(0,0,0,.08)` | `rgba(0,0,0,.6)` | Only if a shadow is genuinely needed |
| `scrim` | `rgba(0,0,0,.4)` | `rgba(0,0,0,.7)` | Modal/drawer backdrop |

These values are a reference, not a mandate when the target brand requires a different palette. Preserve the convention: define both light and dark values for every semantic color, including any accent or status colors the target needs. Keep any CSS fallback map in sync with the canonical token definition. Prefer CSS variables in styles; for canvas or other JS-only rendering, read from the same theme source rather than duplicating values.

Follow the target's existing theme preference behavior. If none exists, system `prefers-color-scheme` is a good default; do not add a manual toggle unless requested. Check contrast in both schemes and do not rely on color alone to convey state.

## Type and spacing

- Use a clear three-role hierarchy: expressive display face for major headings, monospaced face for navigation/code/technical labels, and a highly legible sans-serif for prose. If the target has existing fonts, map these roles to them rather than adding fonts automatically.
- Prefer local font assets where practical; avoid introducing remote font dependencies without a reason.
- Use a fluid root size around `clamp(16px, 14.5px + 0.25vw, 18px)` and size the layout in `rem` so it scales coherently.
- Use a consistent spacing rhythm near `.25, .5, .9, 1.25, 1.5, 2, 2.5rem`. Keep prose comfortable and line lengths bounded; a general content container around `72rem` is a useful upper limit, not a fixed requirement.
- Typical line-height roles: `1.6` for UI and labels, `1.7` for body/card copy, `1.8` for long-form prose. Headings should be tighter, generally `1.15–1.5`.
- Use `clamp()` for display typography so large headings remain deliberate on both narrow and wide viewports.

## Components and layout patterns

- Navigation: compact, centered pill or similarly unobtrusive control; use clear active state and visible keyboard focus. Keep the navigation usable at all viewport widths.
- Buttons: use high-contrast inverted primary actions and quiet outlined/neutral secondary actions. Give interactive controls clear hover, active, disabled, and focus-visible states.
- Cards: simple surface, thin semantic border, restrained radius, useful internal spacing. Use an auto-fit grid (for example `repeat(auto-fit, minmax(15.5rem, 1fr))`) when card content benefits from columns.
- Code/commands: use the mono role, a quiet bordered surface, and horizontal overflow rather than forcing long commands to wrap unpredictably. Copy controls need accessible labels and clear success feedback.
- Documentation/content: prioritize readable prose, consistent heading rhythm, unobtrusive dividers, and keyboard-usable anchors/navigation. Adapt sidebars or drawers to the target's content length and breakpoints.
- Responsive behavior should be intentional: prevent horizontal page overflow, preserve touch target usability, account for safe-area insets when controls touch device edges, and ensure fixed UI does not obscure content.

## Accessibility and implementation checks

- Preserve semantic HTML, meaningful link/button distinctions, labels for icon-only controls, keyboard operation, visible focus, and sufficient light/dark contrast.
- Support `prefers-reduced-motion`; avoid hiding meaningful content behind animation or font loading.
- Reserve image dimensions when known to prevent layout shift; provide appropriate alternative text.
- Test narrow and wide layouts, both color schemes, keyboard focus, and reduced motion. Run the target project's existing lint/build or validation commands when changes are made.

## Suggested workflow

1. Read project instructions and identify existing tokens, type styles, layout primitives, and constraints.
2. Identify the smallest set of changes needed to express this visual language in the target site's existing architecture.
3. Define or map semantic color tokens first; verify both schemes and keep all fallback definitions synchronized.
4. Apply typography, spacing, surfaces, and responsive patterns consistently rather than styling isolated elements ad hoc.
5. Verify accessibility and responsive states, then run the project's checks. Summarize any deliberate deviations from the reference direction.
