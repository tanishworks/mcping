import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { AsciiReveal, AsciiSkeleton } from 'ascii-reveal';
import { Check, Copy } from 'lucide-react';
import { useFontsReady } from '../hooks/useFontsReady';
import { SpiderMascot } from '../components/SpiderMascot';
import iconUrl from '../assets/icon.png';

const REPO_URL = 'https://github.com/tanishworks/mcping';
const RELEASES_URL = `${REPO_URL}/releases`;

// Latest-download URLs are version-independent (release assets carry the
// platform target, not the tag), so this link never goes stale.
const INSTALL_CMD =
    'curl -LO https://github.com/tanishworks/mcping/releases/latest/download/mcping-<target>.tar.gz';

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

    // The reveal wrapper IS the flex column (the .cards-grid pattern), so the
    // install pill and action buttons ride the hero's decode wave instead of
    // popping in stock — their `data-ascii` attributes are inert without it.
    return (
        <AsciiReveal preset="decode" as="div" className="hero__actions">
            <div className="install-pill" data-ascii="fade">
                <code className="install-pill__cmd" data-ascii-digits>
                    <span className="install-pill__prompt">$ </span>
                    {INSTALL_CMD}
                </code>
                <button
                    type="button"
                    className={`install-pill__copy${copied ? ' is-copied' : ''}`}
                    onClick={copy}
                    aria-label={copied ? 'Copied' : 'Copy download command'}
                >
                    {copied ? (
                        <Check size={16} strokeWidth={1.6} aria-hidden />
                    ) : (
                        <Copy size={16} strokeWidth={1.6} aria-hidden />
                    )}
                </button>
            </div>
            <div className="hero__links">
                <a
                    className="btn"
                    href={RELEASES_URL}
                    target="_blank"
                    rel="noreferrer"
                    data-ascii="fade"
                >
                    releases ↗
                </a>
                <Link
                    className="btn btn--secondary"
                    to="/about"
                    data-ascii="fade"
                >
                    how to use?
                </Link>
            </div>
        </AsciiReveal>
    );
}

export function Home() {
    // The whole page renders only once the local fonts are actually loaded:
    // ascii-reveal then decodes real text in its final typeface, so there is
    // no fallback→real font swap mid-animation.
    const fontsReady = useFontsReady();

    return (
        <main className="home">
            {/* ASCII spider in the top-right corner; plays while the fonts (and the
             * hero reveal) get ready, then just keeps spinning there. */}
            <SpiderMascot />
            {fontsReady ? (
                <section className="hero">
                    <AsciiReveal preset="decode" duration="cinematic">
                        {/* App icon (asset carries its own rounded-square shape). */}
                        <img
                            className="hero__icon"
                            src={iconUrl}
                            alt=""
                            width={256}
                            height={256}
                            data-ascii="fade"
                        />
                        <p className="hero__eyebrow" data-ascii="fade">
                            minecraft · java edition · cli
                        </p>
                        <h1 className="hero__title">mcping</h1>
                    </AsciiReveal>
                    <AsciiReveal preset="decode">
                        <p className="hero__tagline" data-ascii="fade">
                            Server status, player counts and latency for any
                            Java Edition server — without ever joining it.
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
        </main>
    );
}
