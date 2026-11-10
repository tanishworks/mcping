import { useCallback, useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useFontsReady } from '../hooks/useFontsReady';
import { useHideOnScroll } from '../hooks/useHideOnScroll';
import { GitHubIcon, IssueIcon, MenuIcon, ReleasesIcon } from './Icons';

const REPO_URL = 'https://github.com/tanishworks/mcping';

const links = [
  { to: '/', label: 'home' },
  { to: '/docs', label: 'docs' },
];

/* Hide/show tuning: the bar may only hide once the page is scrolled past
 * `threshold` px, and `tolerance` px of sub-threshold jitter never flips it. */
const HIDE_THRESHOLD_PX = 90;
const HIDE_TOLERANCE_PX = 6;

/* Same breakpoint as the docs-drawer CSS: below it the bar docks to the
 * bottom edge, so the hover/reveal zone is the bottom edge. */
const BOTTOM_BAR_QUERY = '(max-width: 959px)';
const HOVER_ZONE_PX = 72;

/**
 * Fixed header: a centered pill nav; on /docs the hamburger pops out of the
 * pill's left edge (spring animation) and rests just to its left — it is
 * absolutely positioned inside the cluster, so the pill keeps its exact size
 * and screen position whether the button is there or not. Repo links sit in
 * a solid pill at the right edge.
 *
 * The bar itself is hidden/shown by `useHideOnScroll` (rAF-throttled scroll
 * direction + hover-zone hold); the wrapper's class flips between the base
 * class and `site-headroom--unpinned`. Every real reveal — initial mount
 * included — replays the hamburger pop-out; hover-holds never do.
 */
export function SiteHeader({ onToggleDocs }: { onToggleDocs?: () => void }) {
  const fontsReady = useFontsReady();
  const { pathname } = useLocation();
  const { hidden, hold } = useHideOnScroll({
    threshold: HIDE_THRESHOLD_PX,
    tolerance: HIDE_TOLERANCE_PX,
  });

  // Outer bar wrapper.
  const barRef = useRef<HTMLDivElement | null>(null);

  // Sliding indicator geometry.
  const navRef = useRef<HTMLElement | null>(null);
  const itemRefs = useRef<Record<string, HTMLAnchorElement | null>>({});
  const [indicator, setIndicator] = useState({ x: 0, w: 0, ready: false });

  const measure = useCallback(() => {
    const item = itemRefs.current[pathname];
    if (!item || !navRef.current) return;
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

  // Hamburger pop-out replay: each real reveal (hidden → visible) pops it.
  // Adjusting during render (the documented derived-state pattern) keeps
  // effects free of setState; the initial mount pop comes from the JSX's
  // `is-popping` class, so only later transitions bump the key here.
  const [popKey, setPopKey] = useState(0);
  const [prevHidden, setPrevHidden] = useState(hidden);
  if (prevHidden !== hidden) {
    setPrevHidden(hidden);
    if (!hidden) setPopKey((key) => key + 1);
  }

  // Pointer reveal zones: top edge on the top bar, bottom edge on the
  // bottom-docked (narrow-viewport) bar. Inside the zone the bar is revealed
  // once and *held* visible — scrolling can't re-hide it while hovering.
  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      const bottomBar = window.matchMedia(BOTTOM_BAR_QUERY).matches;
      const inZone = bottomBar
        ? e.clientY >= window.innerHeight - HOVER_ZONE_PX
        : e.clientY <= HOVER_ZONE_PX;
      hold(inZone);
    };
    window.addEventListener('mousemove', onMove, { passive: true });
    return () => window.removeEventListener('mousemove', onMove);
  }, [hold]);

  return (
    <div
      ref={barRef}
      className={`site-headroom${hidden ? ' site-headroom--unpinned' : ''}`}
    >
      <header className="site-header">
        <div className="nav-cluster">
          {onToggleDocs && (
            <button
              key={popKey}
              type="button"
              className="icon-btn menu-btn is-popping"
              aria-label="Toggle docs navigation"
              aria-haspopup="true"
              onClick={onToggleDocs}
            >
              <MenuIcon />
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
            {links.map((link) => (
              <Link
                key={link.to}
                ref={(el: HTMLAnchorElement | null) => {
                  itemRefs.current[link.to] = el;
                }}
                to={link.to}
                className={`pill-nav__link${pathname === link.to ? ' is-active' : ''}`}
              >
                {link.label}
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
            <GitHubIcon />
          </a>
          <a
            className="icon-btn"
            href={`${REPO_URL}/issues`}
            target="_blank"
            rel="noreferrer"
            aria-label="Issues"
          >
            <IssueIcon />
          </a>
          <a
            className="icon-btn"
            href={`${REPO_URL}/releases`}
            target="_blank"
            rel="noreferrer"
            aria-label="Releases"
          >
            <ReleasesIcon />
          </a>
        </div>
      </header>
    </div>
  );
}
