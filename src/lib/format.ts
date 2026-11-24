// mIRC formatting parser: control codes -> styled segments. Pure functions.

export interface Segment {
  text: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  mono: boolean;
  fg: string | null;
  bg: string | null;
}

export const MIRC_COLORS = [
  '#ffffff', '#000000', '#00007f', '#009300', '#ff0000', '#7f0000', '#9c009c', '#fc7f00',
  '#ffff00', '#00fc00', '#009393', '#00ffff', '#0000fc', '#ff00ff', '#7f7f7f', '#d2d2d2'
];

interface Style {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  mono: boolean;
  fg: string | null;
  bg: string | null;
}

const initial = (): Style => ({
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  mono: false,
  fg: null,
  bg: null
});

function readDigits(s: string, i: number, max: number, hex: boolean): [number, number] {
  let n = 0;
  let value = 0;
  while (n < max && i < s.length) {
    const c = s[i];
    const d = hex ? parseInt(c, 16) : c >= '0' && c <= '9' ? c.charCodeAt(0) - 48 : NaN;
    if (Number.isNaN(d)) break;
    value = value * (hex ? 16 : 10) + d;
    i += 1;
    n += 1;
  }
  return [n > 0 ? value : -1, i];
}

export function parseFormatting(text: string): Segment[] {
  const segments: Segment[] = [];
  let style = initial();
  let buf = '';
  const flush = () => {
    if (buf) {
      segments.push({ text: buf, ...style });
      buf = '';
    }
  };
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    switch (c) {
      case '\x02':
        flush();
        style.bold = !style.bold;
        i += 1;
        break;
      case '\x1d':
        flush();
        style.italic = !style.italic;
        i += 1;
        break;
      case '\x1f':
        flush();
        style.underline = !style.underline;
        i += 1;
        break;
      case '\x1e':
        flush();
        style.strike = !style.strike;
        i += 1;
        break;
      case '\x11':
        flush();
        style.mono = !style.mono;
        i += 1;
        break;
      case '\x16':
        flush();
        [style.fg, style.bg] = [style.bg, style.fg];
        i += 1;
        break;
      case '\x0f':
        flush();
        style = initial();
        i += 1;
        break;
      case '\x03': {
        flush();
        i += 1;
        const [fg, next] = readDigits(text, i, 2, false);
        i = next;
        if (fg >= 0) {
          style.fg = MIRC_COLORS[fg % 16];
          if (text[i] === ',') {
            const [bg, next2] = readDigits(text, i + 1, 2, false);
            if (bg >= 0) {
              style.bg = MIRC_COLORS[bg % 16];
              i = next2;
            }
          }
        } else {
          style.fg = null;
          style.bg = null;
        }
        break;
      }
      case '\x04': {
        flush();
        i += 1;
        const [fg, next] = readDigits(text, i, 6, true);
        i = next;
        if (fg >= 0) {
          style.fg = `#${fg.toString(16).padStart(6, '0')}`;
          if (text[i] === ',') {
            const [bg, next2] = readDigits(text, i + 1, 6, true);
            if (bg >= 0) {
              style.bg = `#${bg.toString(16).padStart(6, '0')}`;
              i = next2;
            }
          }
        } else {
          style.fg = null;
          style.bg = null;
        }
        break;
      }
      case '\x01':
        i += 1;
        break;
      default:
        buf += c;
        i += 1;
    }
  }
  flush();
  return segments;
}

const URL_RE = /(https?:\/\/[^\s<>"']+)/g;

export interface LinkPart {
  text: string;
  href: string | null;
}

/** Split a plain-text segment into link / non-link parts. */
export function splitLinks(text: string): LinkPart[] {
  const parts: LinkPart[] = [];
  let last = 0;
  for (const m of text.matchAll(URL_RE)) {
    const idx = m.index ?? 0;
    if (idx > last) parts.push({ text: text.slice(last, idx), href: null });
    parts.push({ text: m[0], href: m[0] });
    last = idx + m[0].length;
  }
  if (last < text.length) parts.push({ text: text.slice(last), href: null });
  return parts;
}

/** Plain preview text (strips all formatting), for notifications and session previews. */
export function plainText(text: string): string {
  return parseFormatting(text)
    .map((s) => s.text)
    .join('');
}