import { useCallback, useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useFontsReady } from '../hooks/useFontsReady';
import { GitHubIcon, IssueIcon, MenuIcon, ReleasesIcon } from './Icons';

const REPO_URL = 'https://github.com/tanishworks/mcping';

const links = [
  { to: '/', label: 'home' },
  { to: '/docs', label: 'docs' },
];

const SHOW_AT_TOP_PX = 90;
const HOVER_ZONE_PX = 72;

/**
 * Fixed header: a centered pill nav; on /docs the hamburger pops out of the
 * pill's left edge (spring animation) and rests just to its left — it is
 * absolutely positioned inside the cluster, so the pill keeps its exact size
 * and screen position whether the button is there or not. Repo links sit in
 * the right column.
 *
 * Behavior:
 * - The pill indicator slides between tabs (measured from the real DOM, so
 *   it tracks font loading and resizes).
 * - The bar hides while scrolling down and reappears on scroll-up or when
 *   the pointer enters the top edge of the screen; each re-entrance replays
 *   the hamburger pop-out.
 */
export function SiteHeader({ onToggleDocs }: { onToggleDocs?: () => void }) {
  const [hidden, setHidden] = useState(false);
  const fontsReady = useFontsReady();
  const { pathname } = useLocation();

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

  // The hamburger pops out of the pill on first mount of the bar and every
  // time the bar comes back (key bump remounts the button → CSS replay).
  // "was-hidden" bookkeeping lives in a ref so the pop only fires on the
  // hidden→visible transition, not on every scroll tick.
  const [popKey, setPopKey] = useState(0);
  const wasHidden = useRef(false);

  const reveal = () => {
    if (wasHidden.current) {
      wasHidden.current = false;
      setHidden(false);
      setPopKey((key) => key + 1);
    }
  };

  const conceal = () => {
    wasHidden.current = true;
    setHidden(true);
  };

  // Hide on scroll-down, reveal on scroll-up or hover near the top edge.
  useEffect(() => {
    let lastY = window.scrollY;
    const onScroll = () => {
      const y = window.scrollY;
      const delta = y - lastY;
      if (delta > 6 && y > SHOW_AT_TOP_PX) conceal();
      else if (delta < -6 || y <= SHOW_AT_TOP_PX) reveal();
      lastY = y;
    };
    const onMove = (e: MouseEvent) => {
      if (e.clientY <= HOVER_ZONE_PX) reveal();
    };
    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('mousemove', onMove, { passive: true });
    return () => {
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('mousemove', onMove);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <header className={`site-header${hidden ? ' site-header--hidden' : ''}`}>
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
  );
}
