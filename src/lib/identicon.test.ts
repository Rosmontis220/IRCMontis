import { describe, expect, it } from 'vitest';
import { channelAvatarUrl, identiconSvg, identiconUrl } from './identicon';

describe('identicon', () => {
  it('is deterministic per nick', () => {
    expect(identiconSvg('alice')).toBe(identiconSvg('alice'));
  });

  it('differs between nicks', () => {
    expect(identiconSvg('alice')).not.toBe(identiconSvg('bob'));
  });

  it('url is an svg data uri', () => {
    expect(identiconUrl('alice').startsWith('data:image/svg+xml')).toBe(true);
  });

  it('channel avatar is deterministic and distinct from nick avatar', () => {
    expect(channelAvatarUrl('#libera')).toBe(channelAvatarUrl('#libera'));
    expect(channelAvatarUrl('#libera')).not.toBe(identiconUrl('#libera'));
  });
});