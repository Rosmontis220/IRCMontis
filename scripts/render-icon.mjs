import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';

const svg = readFileSync(new URL('../static/icon.svg', import.meta.url), 'utf8');
mkdirSync(new URL('../.icon-preview/', import.meta.url), { recursive: true });
for (const size of [512, 128, 32]) {
  const png = new Resvg(svg, { fitTo: { mode: 'width', value: size } }).render().asPng();
  writeFileSync(new URL(`../.icon-preview/icon-${size}.png`, import.meta.url), png);
}
console.log('icon previews written to .icon-preview/');