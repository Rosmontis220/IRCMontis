//! Deterministic geometric avatar (identicon) from a nick. Pure functions.

function fnv1a(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

export function identiconSvg(nick: string): string {
  const h = fnv1a(nick.toLowerCase());
  const hue = h % 360;
  const fg = `hsl(${hue}, 55%, 45%)`;
  const bg = `hsl(${hue}, 45%, 92%)`;
  let cells = '';
  for (let y = 0; y < 5; y++) {
    for (let x = 0; x < 3; x++) {
      const bit = (h >>> (y * 3 + x)) & 1;
      if (!bit) continue;
      const rects = x === 2 ? [[2, y]] : [[x, y], [4 - x, y]];
      for (const [cx, cy] of rects) {
        cells += `<rect x="${cx}" y="${cy}" width="1" height="1"/>`;
      }
    }
  }
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 5" shape-rendering="crispEdges"><rect width="5" height="5" fill="${bg}"/><g fill="${fg}">${cells}</g></svg>`;
}

export function identiconUrl(nick: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(identiconSvg(nick))}`;
}

export function channelAvatarSvg(channel: string): string {
  const h = fnv1a(channel.toLowerCase());
  const hue = h % 360;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40"><rect width="40" height="40" rx="8" fill="hsl(${hue}, 50%, 88%)"/><text x="20" y="27" font-family="Segoe UI, sans-serif" font-size="20" font-weight="600" text-anchor="middle" fill="hsl(${hue}, 55%, 40%)">#</text></svg>`;
}

export function channelAvatarUrl(channel: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(channelAvatarSvg(channel))}`;
}