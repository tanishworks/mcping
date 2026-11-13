import { useEffect, useState, type ReactNode } from 'react';

const FIRST_HEADING_Y = 140;

/**
 * Scroll-spy for the docs sidebar: returns the id of the heading currently
 * under the top of the viewport. When no heading has been reached yet it
 * returns the first heading so the outline starts highlighted.
 */
export function useScrollSpy(headingIds: string[]): string | null {
    const [active, setActive] = useState<string | null>(null);

    useEffect(() => {
        let raf = 0;
        const update = () => {
            raf = 0;
            const content = document.querySelector<HTMLElement>(
                '[data-docs-content]',
            );
            if (!content) return;
            const headings = Array.from(
                content.querySelectorAll<HTMLElement>('h1[id], h2[id], h3[id]'),
            );
            let current: string | null = null;
            for (const heading of headings) {
                if (heading.getBoundingClientRect().top < FIRST_HEADING_Y) {
                    current = heading.id;
                }
            }
            setActive(current);
        };
        const listener = () => {
            if (!raf) raf = requestAnimationFrame(update);
        };
        window.addEventListener('scroll', listener, { passive: true });
        update();
        return () => {
            window.removeEventListener('scroll', listener);
            cancelAnimationFrame(raf);
        };
        // Re-attach when the set of headings changes.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [headingIds.join('|')]);

    return active ?? headingIds[0] ?? null;
}

const slugify = (text: string): string =>
    text
        .toLowerCase()
        .replace(/[^\w\s-]/g, '')
        .trim()
        .replace(/\s+/g, '-');

/** Flattens a ReactNode tree (from react-markdown children) to plain text. */
export function toHeadingText(node: ReactNode): string {
    if (typeof node === 'string' || typeof node === 'number') {
        return String(node);
    }
    if (Array.isArray(node)) return node.map(toHeadingText).join('');
    if (
        node &&
        typeof node === 'object' &&
        'props' in node &&
        (node as { props: { children?: ReactNode } }).props
    ) {
        return toHeadingText(
            (node as { props: { children?: ReactNode } }).props.children,
        );
    }
    return '';
}

export { slugify };
