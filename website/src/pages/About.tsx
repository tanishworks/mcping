import { Link } from 'react-router-dom';

const REPO_URL = 'https://github.com/tanishworks/mcping';
const RELEASES_URL = `${REPO_URL}/releases`;

const platforms = [
  ['macOS · Apple silicon', 'aarch64-apple-darwin'],
  ['macOS · Intel', 'x86_64-apple-darwin'],
  ['Linux · x86_64', 'x86_64-unknown-linux-gnu'],
  ['Linux · ARM64', 'aarch64-unknown-linux-gnu'],
  ['Windows · x86_64', 'x86_64-pc-windows-msvc'],
];

const commands = [
  {
    command: 'mcping status <host[:port]>',
    description: 'Show the server MOTD, version, player count, sample player names, and ping.',
  },
  {
    command: 'mcping ping <host[:port]>',
    description: 'Measure a real ping/pong round-trip to the server.',
  },
  {
    command: 'mcping watch server <host[:port]>',
    description: 'Watch for the server coming online or going offline.',
  },
  {
    command: 'mcping watch player <host[:port]> [name] [--leave]',
    description: 'Watch one player for joins (and leaves with --leave), or omit the name to watch everyone.',
  },
  {
    command: 'mctui <host[:port]> [--interval 5]',
    description: 'Open the live terminal dashboard with a player gauge, ping history, MOTD, and roster.',
  },
];

export function About() {
  return (
    <main className="about-page">
      <header className="about-page__intro">
        <p className="about-page__eyebrow">minecraft · java edition · cli</p>
        <h1 className="about-page__title">About mcping</h1>
        <p className="about-page__lead">
          A small command-line toolkit for checking Minecraft Java servers,
          tracking changes, and keeping a live dashboard in your terminal.
          Get useful status without ever joining the server.
        </p>
      </header>

      <section className="about-page__section" aria-labelledby="about-features">
        <h2 id="about-features">What it does</h2>
        <div className="about-page__feature-grid">
          <article className="about-page__feature">
            <h3>Server status</h3>
            <p>
              Read the MOTD, Minecraft version, online and maximum players,
              any player names the server publishes, and measured latency.
            </p>
          </article>
          <article className="about-page__feature">
            <h3>Watch for changes</h3>
            <p>
              Poll server availability or player activity. Events print in the
              terminal with a bell and can also send a desktop notification.
            </p>
          </article>
          <article className="about-page__feature">
            <h3>Live dashboard</h3>
            <p>
              Use <code>mctui</code> for a full-screen view with a player gauge,
              ping-history chart, MOTD, and scrollable player list.
            </p>
          </article>
          <article className="about-page__feature">
            <h3>Mock server for testing</h3>
            <p>
              <code>mcmock</code> can simulate player joins, leaves, and server
              availability changes while you test scripts and alerts.
            </p>
          </article>
        </div>
      </section>

      <section className="about-page__section" aria-labelledby="about-install">
        <h2 id="about-install">Install</h2>
        <p>
          Download the archive for your operating system and processor from the
          latest release. Each archive includes <code>mcping</code>,{' '}
          <code>mctui</code>, and <code>mcmock</code>; no Rust installation is
          needed to use the prebuilt tools.
        </p>
        <ul className="about-page__platforms">
          {platforms.map(([label, target]) => (
            <li key={target}>
              <span>{label}</span>
              <code>{target}</code>
            </li>
          ))}
        </ul>
        <p>
          Extract the archive and run the binary from its folder, or add that
          folder to your <code>PATH</code>. Windows downloads are ZIP files;
          macOS and Linux downloads are <code>.tar.gz</code> archives.
        </p>
        <a className="btn about-page__release-link" href={RELEASES_URL} target="_blank" rel="noreferrer">
          download a release ↗
        </a>
      </section>

      <section className="about-page__section" aria-labelledby="about-commands">
        <h2 id="about-commands">How to use it</h2>
        <dl className="about-page__commands">
          {commands.map(({ command, description }) => (
            <div className="about-page__command" key={command}>
              <dt><code>{command}</code></dt>
              <dd>{description}</dd>
            </div>
          ))}
        </dl>
        <p className="about-page__hint">
          Addresses can be <code>host</code>, <code>host:port</code>, or{' '}
          <code>[ipv6]:port</code>; the default port is <code>25565</code>.
          Watch commands poll every 10 seconds by default. Set{' '}
          <code>--interval &lt;seconds&gt;</code> to change that,{' '}
          <code>--no-notify</code> for terminal-only events, and press{' '}
          <code>Ctrl-C</code> to stop.
        </p>
      </section>

      <section className="about-page__section" aria-labelledby="about-notes">
        <h2 id="about-notes">A couple of details</h2>
        <ul className="about-page__notes">
          <li>
            Status checks use the server list ping protocol and a real
            ping/pong round-trip. They never send a player join.
          </li>
          <li>
            Player names come from the server&apos;s status sample. Vanilla
            servers may show only a small random sample, and some servers hide
            names entirely.
          </li>
          <li>
            Watch mode requires two consecutive failed polls before reporting
            offline, which helps avoid false alarms from a single hiccup.
          </li>
        </ul>
      </section>

      <nav className="about-page__footer" aria-label="More mcping resources">
        <Link to="/">← home</Link>
        <Link to="/docs">full documentation</Link>
        <a href={REPO_URL} target="_blank" rel="noreferrer">source ↗</a>
      </nav>
    </main>
  );
}
