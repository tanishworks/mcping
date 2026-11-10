import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { AsciiReveal, AsciiSkeleton } from 'ascii-reveal';
import { useFontsReady } from '../hooks/useFontsReady';
import { CheckIcon, CopyIcon } from '../components/Icons';

const INSTALL_CMD = 'git clone https://github.com/tanishworks/mcping && cd mcping && cargo build --release';
const REPO_URL = 'https://github.com/tanishworks/mcping';

const cards = [
  {
    index: '01',
    title: 'Status without joining',
    body: 'MOTD, version, player counts, player names and latency — measured with one real ping/pong round-trip. No join is ever sent to the server.',
  },
  {
    index: '02',
    title: 'Watch modes',
    body: 'Watch a server for online/offline flips or a player for joins and leaves. Events ring the terminal bell and fire desktop notifications.',
  },
  {
    index: '03',
    title: 'Live TUI dashboard',
    body: 'mctui keeps a player gauge, ping-history chart, MOTD and scrollable roster on screen, refreshing in place until you quit.',
  },
];

function InstallPill() {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(INSTALL_CMD);
      setCopied(true);
      timer.current = window.setTimeout(() => setCopied(false), 1600);
    } catch {
      // Clipboard unavailable — leave state as-is.
    }
  };

  return (
    <div className="install-pill" data-ascii="fade">
      <code className="install-pill__cmd" data-ascii-digits>
        <span className="install-pill__prompt">$ </span>
        {INSTALL_CMD}
      </code>
      <button
        type="button"
        className={`install-pill__copy${copied ? ' is-copied' : ''}`}
        onClick={copy}
        aria-label={copied ? 'Copied' : 'Copy build command'}
      >
        {copied ? <CheckIcon /> : <CopyIcon />}
      </button>
    </div>
  );
}

export function Home() {
  // The whole page renders only once the local fonts are actually loaded:
  // ascii-reveal then decodes real text in its final typeface, so there is
  // no fallback→real font swap mid-animation.
  const fontsReady = useFontsReady();

  return (
    <main className="page">
      {fontsReady ? (
        <section className="hero">
          <AsciiReveal preset="decode" duration="cinematic">
            <p className="hero__eyebrow" data-ascii="fade">
              minecraft · java edition · cli
            </p>
            <h1 className="hero__title">mcping</h1>
          </AsciiReveal>
          <AsciiReveal preset="decode">
            <p className="hero__tagline" data-ascii="fade">
              Server status, player counts and latency for any Java Edition
              server — without ever joining it.
            </p>
          </AsciiReveal>
          <InstallPill />
        </section>
      ) : (
        <section className="hero">
          <div className="hero__skeleton">
            <AsciiSkeleton variant="terminal" lines={3} />
          </div>
        </section>
      )}

      {fontsReady && (
        <>
          <section className="cards">
            <AsciiReveal preset="decode">
              {cards.map((card) => (
                <article className="card" key={card.index} data-ascii="surface">
                  <span className="card__index">{card.index}</span>
                  <h2 className="card__title">{card.title}</h2>
                  <p className="card__body">{card.body}</p>
                </article>
              ))}
            </AsciiReveal>
          </section>

          <section>
            <AsciiReveal preset="decode">
              <pre className="terminal" data-ascii="surface">
                <code>
                  <span className="terminal__prompt">$ </span>mcping status alwto.net{'\n'}
                  <span className="terminal__dim">1.21 + | ᴀʟᴡᴛᴏ.ɴᴇᴛ | 26.3 UPDATE SPEARS{'\n'}ᴀʟᴡᴛᴏ ᴘʀᴀᴄᴛɪᴄᴇ | ᴅᴜᴇʟs✦ғғᴀ✦ᴘᴀʀᴛʏ✦ᴇᴠᴇɴᴛs{'\n'}</span>
                  alwto.net:25565{'\n'}
                  <span className="terminal__dim">version  </span>Velocity 1.7.2-26.3 (protocol 767){'\n'}
                  <span className="terminal__dim">players  </span>332/2555{'\n'}
                  <span className="terminal__dim">ping     </span>35ms
                </code>
              </pre>
            </AsciiReveal>
          </section>
        </>
      )}

      <footer className="footer">
        <span>
          rust · no account · <Link to="/docs">docs</Link> ·{' '}
          <a href={REPO_URL} target="_blank" rel="noreferrer">
            source
          </a>
        </span>
      </footer>
    </main>
  );
}
