import {
    createContext,
    useContext,
    useEffect,
    useMemo,
    useSyncExternalStore,
    type ReactNode,
} from 'react';

export type Scheme = 'light' | 'dark';

/**
 * Single source of truth for every color in the app (pure black/white
 * monochrome). Never hard-code a color in a component or stylesheet — add a
 * token here, even if it is used once, and consume it through the CSS
 * variable (`var(--token)`) or the `useTheme()` hook.
 *
 * The ThemeProvider injects one <style> tag from this object and mirrors the
 * active scheme onto `<html data-theme>`, so CSS and JS always agree.
 */
export const colors = {
    /** Page / solid nav background: pure white or pure black. */
    bg: { light: '#ffffff', dark: '#000000' },
    /** Primary text / nav indicator fill: pure black or pure white. */
    fg: { light: '#000000', dark: '#ffffff' },
    /** Secondary text. */
    muted: { light: '#5e5e5e', dark: '#a1a1a1' },
    /** Tertiary text (labels, skeletons). */
    faint: { light: '#9c9c9c', dark: '#575757' },
    /** 1px separators and card borders. */
    hairline: {
        light: 'rgba(0, 0, 0, 0.14)',
        dark: 'rgba(255, 255, 255, 0.16)',
    },
    /** Subtle interactive backgrounds (hover fills, inline code). */
    hover: { light: 'rgba(0, 0, 0, 0.05)', dark: 'rgba(255, 255, 255, 0.07)' },
    /** Moving gradient blobs (the only grays allowed to breathe). */
    gradientA: { light: '#e7e7e7', dark: '#161616' },
    gradientB: { light: '#f2f2f2', dark: '#212121' },
    gradientC: { light: '#d6d6d6', dark: '#2b2b2b' },
    /** Opacity of the noise grain layered over the gradient background. */
    noise: { light: '0.045', dark: '0.08' },
    /** Box shadows. */
    shadow: { light: 'rgba(0, 0, 0, 0.08)', dark: 'rgba(0, 0, 0, 0.6)' },
    /** Dark scrim behind the mobile docs drawer. */
    scrim: { light: 'rgba(0, 0, 0, 0.4)', dark: 'rgba(0, 0, 0, 0.7)' },
    /** Brief per-character flash color used by ascii-reveal. */
    flash: { light: '#ffffff', dark: '#0d0d0d' },
} as const;

export type ColorToken = keyof typeof colors;

const toVarName = (token: string): string =>
    `--${token.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`;

/** CSS declarations for a single scheme; used to build the injected styles. */
function schemeCss(scheme: Scheme): string {
    const lines = Object.entries(colors).map(
        ([token, pair]) => `  ${toVarName(token)}: ${pair[scheme]};`,
    );
    return `  color-scheme: ${scheme};\n${lines.join('\n')}`;
}

const themeStyle = `
:root {
${schemeCss('light')}
}
:root[data-theme='dark'] {
${schemeCss('dark')}
}
`;

// The active scheme follows the OS preference via matchMedia, so switching
// the system theme restyles the site live (no manual toggle needed).
const darkQuery =
    typeof window !== 'undefined'
        ? window.matchMedia('(prefers-color-scheme: dark)')
        : null;

function subscribe(onChange: () => void): () => void {
    darkQuery?.addEventListener('change', onChange);
    return () => darkQuery?.removeEventListener('change', onChange);
}

function getScheme(): Scheme {
    return darkQuery?.matches ? 'dark' : 'light';
}

function getServerScheme(): Scheme {
    return 'light';
}

interface ThemeValue {
    scheme: Scheme;
    /** CSS variable reference, e.g. `var(--bg)`. Prefer this in styles. */
    varName: (token: ColorToken) => string;
    /** Resolved value for the active scheme — only for canvas/JS work. */
    color: (token: ColorToken) => string;
}

const ThemeContext = createContext<ThemeValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
    const scheme = useSyncExternalStore(subscribe, getScheme, getServerScheme);

    useEffect(() => {
        document.documentElement.dataset.theme = scheme;
    }, [scheme]);

    const value: ThemeValue = useMemo(
        () => ({
            scheme,
            varName: (t) => `var(${toVarName(t)})`,
            color: (t) => colors[t][scheme],
        }),
        [scheme],
    );

    return (
        <ThemeContext.Provider value={value}>
            <style
                id="app-theme"
                dangerouslySetInnerHTML={{ __html: themeStyle }}
            />
            {children}
        </ThemeContext.Provider>
    );
}

/** Hook to read the current scheme and theme colors. */
export function useTheme(): ThemeValue {
    const ctx = useContext(ThemeContext);
    if (!ctx) throw new Error('useTheme must be used inside <ThemeProvider>');
    return ctx;
}
