import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { Check, Copy } from 'lucide-react';
import { SpiderMascot } from '../components/SpiderMascot';

// Keep this copy-ready subset in sync with src/content/docs.md → Common tasks.
const examples = [
  {
    title: 'See server status',
    description: 'Get the MOTD, version, player count, sample names, and latency.',
    command: 'mcping status alwto.net',
  },
  {
    title: 'Check latency',
    description: 'Measure one real ping/pong round-trip.',
    command: 'mcping ping alwto.net',
  },
  {
    title: 'Know when a server changes',
    description: 'Watch for online/offline changes; Ctrl-C stops the watcher.',
    command: 'mcping watch server alwto.net --interval 30',
  },
  {
    title: 'Track a player',
    description: 'Get join and leave alerts for one player.',
    command: 'mcping watch player alwto.net D1xP --leave',
  },
  {
    title: 'Watch everyone',
    description: 'Omit the player name to report joins from the published sample.',
    command: 'mcping watch player alwto.net',
  },
  {
    title: 'Open the live dashboard',
    description: 'View the player gauge, ping history, MOTD, and roster. Press q to quit.',
    command: 'mctui alwto.net --interval 5',
  },
];

function CopyCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      timer.current = window.setTimeout(() => setCopied(false), 1600);
    } catch {
      // Clipboard unavailable — keep the command visible for manual copy.
    }
  };

  return (
    <div className="about-page__command">
      <code>{command}</code>
      <button
        type="button"
        className="about-page__copy"
        onClick={copy}
        aria-label={copied ? 'Command copied' : 'Copy command'}
        title={copied ? 'Copied' : 'Copy command'}
      >
        {copied ? <Check size={16} strokeWidth={1.6} aria-hidden /> : <Copy size={16} strokeWidth={1.6} aria-hidden />}
      </button>
    </div>
  );
}

export function About() {
  return (
    <main className="about-page">
      {/* ASCII spider in the top-right corner, same as the home page. */}
      <SpiderMascot />
      <header className="about-page__intro">
        <p className="about-page__eyebrow">mcping · quick examples</p>
        <h1 className="about-page__title">Explore mcping</h1>
        <p className="about-page__lead">
          Already installed? Choose what you want to do, copy the command, and
          paste it into your terminal. These examples use <code>alwto.net</code>;
          replace it with your server address.
        </p>
      </header>

      <section className="about-page__examples" aria-label="Common tasks">
        {examples.map(({ title, description, command }) => (
          <article className="about-page__example" key={command}>
            <h2>{title}</h2>
            <p>{description}</p>
            <CopyCommand command={command} />
          </article>
        ))}
      </section>

      <p className="about-page__hint">
        Use <code>host:port</code> for a custom port (default <code>25565</code>).
        Watch commands poll every 10 seconds by default; use{' '}
        <code>--interval &lt;seconds&gt;</code> to change it. Player names depend
        on what the server publishes. See <Link to="/docs">docs</Link> for
        installation, all options, and full details.
      </p>
    </main>
  );
}
