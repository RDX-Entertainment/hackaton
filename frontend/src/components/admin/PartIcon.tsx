import type { ReactNode } from 'react';

import type { ComponentKind } from '../../utils/api';

type PartIconProps = {
  kind: ComponentKind;
  className?: string;
};

const PATHS: Record<ComponentKind, ReactNode> = {
  mobo: (
    <>
      <rect x="2.5" y="2.5" width="19" height="19" rx="1" />
      <rect x="5.5" y="5.5" width="6" height="6" rx="0.5" />
      <path d="M7 8h3M8.5 6.5v3" />
      <path d="M15 5.5h4M15 8h4M15 10.5h4" />
      <path d="M5.5 15.5h13M5.5 18h8" />
      <circle cx="19" cy="18" r="1.2" />
    </>
  ),
  cpu: (
    <>
      <rect x="6.5" y="6.5" width="11" height="11" rx="1" />
      <rect x="10" y="10" width="4" height="4" rx="0.5" />
      <path d="M10 3.5v3M14 3.5v3M10 17.5v3M14 17.5v3" />
      <path d="M3.5 10h3M3.5 14h3M17.5 10h3M17.5 14h3" />
    </>
  ),
  ram: (
    <>
      <path d="M2 7.5h20v9H2z" />
      <rect x="5" y="10" width="3" height="4" rx="0.4" />
      <rect x="10.5" y="10" width="3" height="4" rx="0.4" />
      <rect x="16" y="10" width="3" height="4" rx="0.4" />
      <path d="M5 16.5v2M8 16.5v2M11 16.5v2M14 16.5v2M17 16.5v2" />
      <path d="M10 18.5h4" />
    </>
  ),
  hdd: (
    <>
      <rect x="2.5" y="5.5" width="19" height="13" rx="1" />
      <circle cx="11" cy="12" r="4" />
      <circle cx="11" cy="12" r="1" />
      <path d="M17.5 8.5 13.8 14.5" />
      <circle cx="18" cy="8" r="1" />
    </>
  ),
  net: (
    <>
      <path d="M8 3.5h8v5h3v12H5v-12h3z" />
      <path d="M8 12v4M10.5 12v4M13.5 12v4M16 12v4" />
      <path d="M11 3.5v2M13 3.5v2" />
    </>
  ),
};

export function PartIcon({ kind, className }: PartIconProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[kind]}
    </svg>
  );
}
