/*
 * Game addresses travel as the 32-bit integer the IPv4 packs into, which is how
 * the original stored them with ip2long(). Everything the player sees or types
 * is dotted quad, so the conversion lives here and nowhere else.
 */

const OCTETS = 4;
const MAX_OCTET = 255;
const MAX_IP = 4294967295;

/** 3232235777 → "192.168.1.1". */
export function formatIp(value: number | string | null | undefined): string {
  const numeric = Number(value);

  if (!Number.isFinite(numeric) || numeric < 0 || numeric > MAX_IP) {
    return '—';
  }

  const packed = Math.trunc(numeric);

  return [
    // Not `>>>`: it works here, but the intent is arithmetic on a 32-bit value,
    // and division states that without depending on JS bitwise coercion.
    Math.floor(packed / 16777216) & MAX_OCTET,
    Math.floor(packed / 65536) & MAX_OCTET,
    Math.floor(packed / 256) & MAX_OCTET,
    packed & MAX_OCTET,
  ].join('.');
}

/** "192.168.1.1" → 3232235777. Returns null when it is not an address. */
export function parseIp(text: string): number | null {
  const parts = text.trim().split('.');

  if (parts.length !== OCTETS) {
    return null;
  }

  let packed = 0;

  for (const part of parts) {
    if (!/^\d{1,3}$/.test(part)) {
      return null;
    }

    const octet = Number(part);
    if (octet > MAX_OCTET) {
      return null;
    }

    packed = packed * 256 + octet;
  }

  return packed;
}

/** True once the text is a complete, valid address. */
export function isIp(text: string): boolean {
  return parseIp(text) !== null;
}

const MASK = '•••';

/**
 * Hides the tail of an address, keeping enough of the head to recognise the
 * network. `172.58.12.9` with two octets visible becomes `172.58.•••.•••`.
 *
 * The masking happens here, on already-fetched data: it is a courtesy for
 * screenshots and shoulder-surfing, not a security boundary. Anything that must
 * not reach the client should not be sent by the API in the first place.
 */
export function maskIp(
  value: number | string | null | undefined,
  visibleOctets = 2,
): string {
  const text = formatIp(value);

  if (text === '—') {
    return text;
  }

  const kept = Math.min(Math.max(visibleOctets, 0), OCTETS);
  const octets = text.split('.');

  return octets.map((octet, index) => (index < kept ? octet : MASK)).join('.');
}
