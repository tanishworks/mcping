import { useEffect, useState } from 'react';

/**
 * Waits until every family we actually use is loaded and reports the moment
 * text can be laid out in its final metrics. Hero/card reveals are gated on
 * this so ascii-reveal never decodes text in a fallback font and then
 * reflows it into the real one.
 */
export function useFontsReady(): boolean {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let alive = true;
    const wanted = [
      '16px Inter',
      '16px "JetBrains Mono"',
      '16px "Press Start 2P"',
    ];
    Promise.all(wanted.map((spec) => document.fonts.load(spec)))
      .then(() => document.fonts.ready)
      .then(() => {
        if (alive) setReady(true);
      });
    return () => {
      alive = false;
    };
  }, []);

  return ready;
}
