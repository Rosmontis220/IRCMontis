import { describe, expect, it } from 'vitest';
import { MIRC_COLORS, parseFormatting, plainText, splitLinks } from './format';

describe('parseFormatting', () => {
  it('plain text yields one unstyled segment', () => {
    expect(parseFormatting('hello')).toEqual([
      { text: 'hello', bold: false, italic: false, underline: false, strike: false, mono: false, fg: null, bg: null }
    ]);
  });

  it('toggles bold and resets on close', () => {
    const segs = parseFormatting('a\x02b\x02c');
    expect(segs.map((s) => [s.text, s.bold])).toEqual([
      ['a', false],
      ['b', true],
      ['c', false]
    ]);
  });

  it('parses mirc foreground and background', () => {
    const [seg] = parseFormatting('\x0304,02x');
    expect(seg.fg).toBe(MIRC_COLORS[4]);
    expect(seg.bg).toBe(MIRC_COLORS[2]);
  });

  it('bare color reset clears fg and bg', () => {
    const segs = parseFormatting('\x0304a\x03b');
    expect(segs[0].fg).toBe(MIRC_COLORS[4]);
    expect(segs[1].fg).toBeNull();
    expect(segs[1].bg).toBeNull();
  });

  it('parses hex colors', () => {
    const [seg] = parseFormatting('\x04ff0000,00ff00x');
    expect(seg.fg).toBe('#ff0000');
    expect(seg.bg).toBe('#00ff00');
  });

  it('reverse swaps fg and bg', () => {
    const [seg] = parseFormatting('\x0304,02a\x16b');
    const segs = parseFormatting('\x0304,02a\x16b');
    expect(segs[1].fg).toBe(MIRC_COLORS[2]);
    expect(segs[1].bg).toBe(MIRC_COLORS[4]);
    expect(seg.text).toBe('a');
  });

  it('reset code clears all styles', () => {
    const segs = parseFormatting('\x02\x1f\x0304a\x0fb');
    expect(segs[1]).toMatchObject({ text: 'b', bold: false, underline: false, fg: null });
  });

  it('monospace toggle', () => {
    const segs = parseFormatting('\x11code\x11');
    expect(segs[0].mono).toBe(true);
  });
});

describe('splitLinks', () => {
  it('splits url out of surrounding text', () => {
    const parts = splitLinks('see https://libera.chat/guide now');
    expect(parts).toEqual([
      { text: 'see ', href: null },
      { text: 'https://libera.chat/guide', href: 'https://libera.chat/guide' },
      { text: ' now', href: null }
    ]);
  });

  it('no url yields single plain part', () => {
    expect(splitLinks('plain')).toEqual([{ text: 'plain', href: null }]);
  });
});

describe('plainText', () => {
  it('strips all control codes', () => {
    expect(plainText('\x02bold\x02 \x0304red\x0f ok')).toBe('bold red ok');
  });
});