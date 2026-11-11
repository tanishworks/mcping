import { useCallback, useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { BookOpen, CircleDot, House, Menu, Tag, Compass } from 'lucide-react';
import { useFontsReady } from '../hooks/useFontsReady';
import { GitHubMark } from './GitHubMark';

const REPO_URL = 'https://github.com/tanishworks/mcping';

/* Shared lucide props: 16px box, 1.6 stroke to match the hairline weight. */
const iconProps = { size: 16, strokeWidth: 1.6, 'aria-hidden': true } as const;

const links = [
    { to: '/', label: 'home', icon: House },
    { to: '/about', label: 'explore', icon: Compass },
    { to: '/docs', label: 'docs', icon: BookOpen },
];

/**
 * Fixed header: a centered pill nav; on /docs the hamburger pops out of the
 * pill's left edge (spring animation) and rests just to its left — it is
 * absolutely positioned inside the cluster, so the pill keeps its exact size
 * and screen position whether the button is there or not. Repo links sit in
 * a solid pill at the right edge.
 *
 * The bar is always visible — no hide-on-scroll logic, deliberately simple.
 * Desktop tabs are text-only; below 960px (the docs-drawer breakpoint) the
 * bar docks to the bottom edge and the tabs become icon + label.
 */
export function SiteHeader({ onToggleDocs }: { onToggleDocs?: () => void }) {
    const fontsReady = useFontsReady();
    const { pathname } = useLocation();

    // Sliding indicator geometry.
    const navRef = useRef<HTMLElement | null>(null);
    const itemRefs = useRef<Record<string, HTMLAnchorElement | null>>({});
    const [indicator, setIndicator] = useState({ x: 0, w: 0, ready: false });

    const measure = useCallback(() => {
        const item = itemRefs.current[pathname];
        if (!navRef.current) return;
        if (!item) {
            setIndicator((current) =>
                current.ready ? { ...current, ready: false } : current,
            );
            return;
        }
        setIndicator({ x: item.offsetLeft, w: item.offsetWidth, ready: true });
    }, [pathname]);

    // Re-measure on route change and once fonts finish loading (text metrics
    // change, which is exactly when the slide would otherwise be off).
    useEffect(() => {
        measure();
    }, [measure, fontsReady]);

    useEffect(() => {
        window.addEventListener('resize', measure);
        return () => window.removeEventListener('resize', measure);
    }, [measure]);

    return (
        <div className="site-headroom">
            <header className="site-header">
                <div className="nav-cluster">
                    {onToggleDocs && (
                        <button
                            type="button"
                            className="icon-btn menu-btn is-popping"
                            aria-label="Toggle docs navigation"
                            aria-haspopup="true"
                            onClick={onToggleDocs}
                        >
                            <Menu {...iconProps} />
                        </button>
                    )}

                    <nav className="pill-nav" aria-label="Primary" ref={navRef} data-ascii="skip">
                        <span
                            className="pill-nav__indicator"
                            style={{
                                transform: `translateX(${indicator.x}px)`,
                                width: indicator.w,
                                opacity: indicator.ready ? 1 : 0,
                            }}
                        />
                        {links.map(({ to, label, icon: Icon }) => (
                            <Link
                                key={to}
                                ref={(el: HTMLAnchorElement | null) => {
                                    itemRefs.current[to] = el;
                                }}
                                to={to}
                                className={`pill-nav__link${pathname === to ? ' is-active' : ''}`}
                            >
                                <Icon className="pill-nav__icon" {...iconProps} />
                                <span className="pill-nav__label">{label}</span>
                            </Link>
                        ))}
                    </nav>
                </div>

                <div className="site-header__side site-header__side--right">
                    <a
                        className="icon-btn"
                        href={REPO_URL}
                        target="_blank"
                        rel="noreferrer"
                        aria-label="GitHub repository"
                    >
                        <GitHubMark />
                    </a>
                    <a
                        className="icon-btn"
                        href={`${REPO_URL}/issues`}
                        target="_blank"
                        rel="noreferrer"
                        aria-label="Issues"
                    >
                        <CircleDot {...iconProps} />
                    </a>
                    <a
                        className="icon-btn"
                        href={`${REPO_URL}/releases`}
                        target="_blank"
                        rel="noreferrer"
                        aria-label="Releases"
                    >
                        <Tag {...iconProps} />
                    </a>
                </div>
            </header>
        </div>
    );
}
