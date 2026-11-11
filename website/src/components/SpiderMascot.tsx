import { useEffect, useRef } from 'react';
import { createAsciiReveal } from 'ascii-reveal/core';

/**
 * Decorative ASCII spider mascot pinned to a page corner.
 *
 * The mascot normally plays inside an `AsciiReveal` while something is
 * loading; here nothing is loading, so the controller from `ascii-reveal/core`
 * is driven directly: its `shimmer` loop is what renders a `mascot`, so a
 * standalone mascot is a controller over an empty element whose shimmer just
 * runs forever. Reduced motion is a no-op inside the library, so nothing
 * draws for those users.
 */
export function SpiderMascot({ side = 'right' }: { side?: 'left' | 'right' }) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ctrl = createAsciiReveal(el, { mascot: 'spider', preset: 'decode' });
    ctrl.shimmer(true);
    return () => ctrl.destroy();
  }, []);

  return (
    <div
      ref={ref}
      className={`ascii-mascot ascii-mascot--${side}`}
      aria-hidden="true"
    />
  );
}
