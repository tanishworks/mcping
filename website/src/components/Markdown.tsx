import type { ComponentPropsWithoutRef, ReactNode } from 'react';
import ReactMarkdown, { type Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { slugify, toHeadingText } from '../hooks/useScrollSpy';

/**
 * Markdown renderer with custom, theme-token styling only (see `.md` rules
 * in styles/global.css). Headings get slug ids so the sidebar outline can
 * scroll-spy and deep-link to them.
 */

type HeadingProps = ComponentPropsWithoutRef<'h2'>;

/** Flattens markdown-react children back to the plain heading text. */
const textOf = (children: ReactNode): string => toHeadingText(children);

function splitNodeProps(props: Record<string, unknown>): Record<string, unknown> {
  // react-markdown may hand every component its AST in `node?`; never
  // forward it to real DOM elements.
  const clean = { ...props };
  delete clean.node;
  return clean;
}

function heading(tag: 'h1' | 'h2' | 'h3') {
  return ({ children, ...rest }: HeadingProps) => {
    const Tag = tag;
    return (
      <Tag
        id={slugify(textOf(children))}
        {...(splitNodeProps(rest) as HeadingProps)}
      >
        {children}
      </Tag>
    );
  };
}

function anchor(props: ComponentPropsWithoutRef<'a'>) {
  const clean = splitNodeProps(props) as ComponentPropsWithoutRef<'a'>;
  const external = typeof clean.href === 'string' && clean.href.startsWith('http');
  return (
    <a
      {...clean}
      target={external ? '_blank' : undefined}
      rel={external ? 'noreferrer' : undefined}
    />
  );
}

function table(props: ComponentPropsWithoutRef<'table'>) {
  return (
    <div className="md__table">
      <table {...(splitNodeProps(props) as ComponentPropsWithoutRef<'table'>)} />
    </div>
  );
}

const components: Components = {
  h1: heading('h1'),
  h2: heading('h2'),
  h3: heading('h3'),
  a: anchor,
  table,
};

export function Markdown({ children }: { children: string }) {
  return (
    <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
      {children}
    </ReactMarkdown>
  );
}
