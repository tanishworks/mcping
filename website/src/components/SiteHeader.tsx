import { useCallback, useEffect, useRef, useState } from 'react';
import Headroom from 'headroom.js';
import { Link, useLocation } from 'react-router-dom';
import { useFontsReady } from '../hooks/useFontsReady';
import { GitHubIcon, IssueIcon, MenuIcon, ReleasesIcon } from './Icons';

const REPO_URL = 'https://github.com/tanishworks/mcping';

const links = [
  { to: '/', label: 'home' },
  { to: '/docs', label: 'docs' },
];

/* Scroll-direction awareness comes from headroom.js (offset in px before the
 * bar may hide, tolerance in px before a direction change counts). The bar
 * tucks up while scrolling down and drops back from the top on scroll-up;
 * CSS owns the transforms (see .site-headroom rules in global.css). */
const HEADROOM_OFFSET = 90;
const HEADROOM_TOLERANCE = 6;

/* Same breakpoint as the docs-drawer CSS: below it the bar docks to the
 * bottom edge, so the hover/reveal zone is the bottom edge. */
const BOTTOM_BAR_QUERY = '(max-width: 959px)';
const HOVER_ZONE_PX = 72;

const HEADROOM_CLASSES = {
  initial: 'site-headroom',
  pinned: 'site-headroom--pinned',
  unpinned: 'site-headroom--unpinned',
  top: 'site-headroom--top',
  notTop: 'site-headroom--not-top',
  bottom: 'site-headroom--bottom',
  notBottom: 'site-headroom--not-bottom',
  frozen: 'site-headroom--frozen',
};

/**
 * Fixed header: a centered pill nav; on /docs the hamburger pops out of the
 * pill's left edge (spring animation) and rests just to its left — it is
 * absolutely positioned inside the cluster, so the pill keeps its exact size
 * and screen position whether the button is there or not. Repo links sit in
 * a solid pill at the right edge.
 *
 * The bar itself is managed by headroom.js running on the outer
 * `.site-headroom` wrapper (never re-rendered by React, so headroom's class
 * list can't be clobbered). Every re-pin — initial mount included — replays
 * the hamburger pop-out.
 */
export function SiteHeader({ onToggleDocs }: { onToggleDocs?: () => void }) {
  const fontsReady = useFontsReady();
  const { pathname } = useLocation();

  // Outer bar wrapper (owns headroom's class list — static in React's eyes).
  const barRef = useRef<HTMLDivElement | null>(null);
  const headroomRef = useRef<Headroom | null>(null);

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

  // Hamburger pop-out replay; fired by headroom's onPin and re-pins.
  const [popKey, setPopKey] = useState(0);
  const replayPop = useCallback(() => setPopKey((key) => key + 1), []);

  // headroom.js lifecycle: hide on scroll down, drop from the top on up.
  useEffect(() => {
    const bar = barRef.current;
    if (!bar) return;
    const headroom = new Headroom(bar, {
      offset: HEADROOM_OFFSET,
      tolerance: HEADROOM_TOLERANCE,
      classes: HEADROOM_CLASSES,
      onPin: replayPop,
    });
    headroom.init();
    headroomRef.current = headroom;
    return () => {
      headroom.destroy();
      headroomRef.current = null;
    };
  }, [replayPop]);

  // Pointer reveal zones: top edge on the top bar, bottom edge on the
  // bottom-docked (narrow-viewport) bar. Forcing `pin()` also replays the pop.
  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      const bottomBar = window.matchMedia(BOTTOM_BAR_QUERY).matches;
      const inZone = bottomBar
        ? e.clientY >= window.innerHeight - HOVER_ZONE_PX
        : e.clientY <= HOVER_ZONE_PX;
      if (inZone) headroomRef.current?.pin();
    };
    window.addEventListener('mousemove', onMove, { passive: true });
    return () => window.removeEventListener('mousemove', onMove);
  }, []);

  return (
    <div ref={barRef} className="site-headroom">
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
