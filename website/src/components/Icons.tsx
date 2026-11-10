import type { SVGProps } from 'react';

/**
 * Icons use `currentColor`/stroke so they inherit the theme color wherever
 * they sit (and stay visible while ascii-reveal hides surrounding text).
 */

const fillProps: SVGProps<SVGSVGElement> = {
  width: 16,
  height: 16,
  viewBox: '0 0 16 16',
  fill: 'currentColor',
  'aria-hidden': true,
};

const strokeProps: SVGProps<SVGSVGElement> = {
  width: 16,
  height: 16,
  viewBox: '0 0 16 16',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.6,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
  'aria-hidden': true,
};

export function GitHubIcon() {
  return (
    <svg {...fillProps}>
      <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.66-.21-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.55-1.03-2.21-.82-2.21-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z" />
    </svg>
  );
}

export function IssueIcon() {
  return (
    <svg {...fillProps}>
      <path d="M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z" />
      <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z" />
    </svg>
  );
}

export function ReleasesIcon() {
  return (
    <svg {...fillProps}>
      <path d="M1 7.775V2.5C1 1.784 1.784 1 2.5 1h5.275c.464 0 .909.184 1.237.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm5.48-4.275H2.5a.75.75 0 0 0-.75.75v5.275c0 .198.079.389.22.53l6.25 6.25a.75.75 0 0 0 1.06 0l5.026-5.026a.75.75 0 0 0 0-1.06l-6.25-6.25a.75.75 0 0 0-.53-.22ZM5.5 5a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z" />
    </svg>
  );
}

export function MenuIcon() {
  return (
    <svg {...strokeProps}>
      <path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" />
    </svg>
  );
}

export function CopyIcon() {
  return (
    <svg {...strokeProps}>
      <rect x="5.5" y="5.5" width="8" height="9" rx="1.5" />
      <path d="M10.5 3.5v-.5c0-.8-.7-1.5-1.5-1.5H4c-.8 0-1.5.7-1.5 1.5v7c0 .8.7 1.5 1.5 1.5h1.5" />
    </svg>
  );
}

export function CheckIcon() {
  return (
    <svg {...strokeProps}>
      <path d="M2.5 8.5l3.5 3.5 7.5-8" />
    </svg>
  );
}
