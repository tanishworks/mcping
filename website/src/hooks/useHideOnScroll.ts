import { useCallback, useEffect, useRef, useState } from 'react';

/**
 * The conventional hide-on-scroll-down / show-on-scroll-up navbar hook.
 *
 * A rAF-throttled (one sample per frame, passive listener) scroll handler
 * compares the scroll position with the last sample: scrolling down past
 * `threshold` px hides the bar, any upward scroll past `tolerance` px
 * reveals it, and the bar is always visible near the top of the page.
 *
 * `hold(held)` is the reveal-zone escape hatch: while the pointer rests in
 * the hover zone the bar is revealed once and *kept* visible — scroll events
 * are not allowed to re-hide it — so hovering can never fight the scroll
 * direction (the flicker forced `pin()` calls used to produce). When the
 * pointer leaves, the next scroll sample resumes normal direction logic.
 */
export function useHideOnScroll({
  threshold = 90,
  tolerance = 6,
}: {
  threshold?: number;
  tolerance?: number;
} = {}): { hidden: boolean; hold: (held: boolean) => void } {
  const [hidden, setHidden] = useState(false);

  // Mirror state in refs so passive listeners read fresh values without
  // being torn down and re-registered on every flip.
  const hiddenRef = useRef(false);
  const heldRef = useRef(false);
  const lastY = useRef(0);

  const setBarHidden = useCallback((next: boolean) => {
    if (hiddenRef.current === next) return;
    hiddenRef.current = next;
    setHidden(next);
  }, []);

  const hold = useCallback(
    (held: boolean) => {
      heldRef.current = held;
      if (held) setBarHidden(false);
    },
    [setBarHidden],
  );

  useEffect(() => {
    lastY.current = window.scrollY;
    let frame = 0;

    const sample = () => {
      frame = 0;
      const y = window.scrollY;
      const delta = y - lastY.current;
      if (Math.abs(delta) <= tolerance) return; // sub-tolerance jitter: no flip
      lastY.current = y;
      if (heldRef.current) return; // pointer owns the bar; scroll may not hide it
      if (y <= threshold) setBarHidden(false); // near the top: always visible
      else if (delta > 0) setBarHidden(true); // scrolling down: tuck away
      else setBarHidden(false); // scrolling up: reveal
    };

    // One sample per frame: skip scheduling while one is already queued.
    const onScroll = () => {
      if (frame) return;
      frame = requestAnimationFrame(sample);
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    return () => {
      window.removeEventListener('scroll', onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, [setBarHidden, threshold, tolerance]);

  return { hidden, hold };
}
