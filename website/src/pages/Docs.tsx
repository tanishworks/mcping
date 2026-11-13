import { useMemo } from 'react';
import { AsciiReveal, AsciiSkeleton } from 'ascii-reveal';
import { useFontsReady } from '../hooks/useFontsReady';
import { slugify, useScrollSpy } from '../hooks/useScrollSpy';
import { Markdown } from '../components/Markdown';
import { SpiderMascot } from '../components/SpiderMascot';
import docsMd from '../content/docs.md?raw';

interface Heading {
    id: string;
    title: string;
    level: 2 | 3;
}

/** Extracts h2/h3 structure from the markdown source for the sidebar outline. */
function parseHeadings(md: string): Heading[] {
    const headings: Heading[] = [];
    let inFence = false;
    for (const line of md.split('\n')) {
        if (line.startsWith('```')) inFence = !inFence;
        if (inFence) continue;
        const match = line.match(/^(#{2,3})\s+(.+)$/);
        if (!match) continue;
        const title = match[2].trim();
        headings.push({
            id: slugify(title),
            title,
            level: match[1].length as 2 | 3,
        });
    }
    return headings;
}

interface DocsProps {
    /** Whether the sidebar is currently shown (drawer on mobile / column on desktop). */
    open: boolean;
    /** Called when the mobile drawer backdrop is clicked. */
    closeSidebar: () => void;
}

export function Docs({ open, closeSidebar }: DocsProps) {
    const headings = useMemo(() => parseHeadings(docsMd), []);
    const activeId = useScrollSpy(headings.map((h) => h.id));
    const fontsReady = useFontsReady();
    const sidebar = `${open ? ' docs--sidebar-open' : ' docs--sidebar-closed'}`;

    if (!fontsReady) {
        return (
            <main className="docs docs--loading">
                <SpiderMascot side="left" />
                <div className="hero__skeleton">
                    <AsciiSkeleton lines={6} />
                </div>
            </main>
        );
    }

    return (
        <div className={`docs${sidebar}`}>
            {/* ASCII spider in the top-left corner; plays while the fonts (and the
             * reveal) get ready, then just keeps spinning there. */}
            <SpiderMascot side="left" />
            {/* Backdrop behind the mobile drawer only; hidden on wide screens. */}
            <div
                className={`docs__scrim${open ? ' is-open' : ''}`}
                onClick={closeSidebar}
                aria-hidden="true"
            />

            <aside className="docs__sidebar">
                <nav aria-label="On this page">
                    <p className="docs__sidebar-label">on this page</p>
                    {headings.map((heading) => (
                        <a
                            key={heading.id}
                            href={`#${heading.id}`}
                            className={[
                                'docs__sidebar-link',
                                `docs__sidebar-link--h${heading.level}`,
                                activeId === heading.id ? 'is-active' : '',
                            ]
                                .filter(Boolean)
                                .join(' ')}
                        >
                            {heading.title}
                        </a>
                    ))}
                </nav>
            </aside>

            <main className="docs__content-wrap">
                <article className="md" data-docs-content>
                    <AsciiReveal preset="decode" lazy={false}>
                        <Markdown>{docsMd}</Markdown>
                    </AsciiReveal>
                </article>
            </main>
        </div>
    );
}
