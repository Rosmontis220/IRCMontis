import { createLightTheme, createDarkTheme, type BrandVariants } from '@fluentui/tokens';
import { setTheme } from '@fluentui/web-components/theme/set-theme.js';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type Scheme = 'light' | 'dark';

const brand: BrandVariants = {
  10: '#001428', 20: '#002445', 30: '#003562', 40: '#00467e',
  50: '#00579b', 60: '#0068b8', 70: '#007ad5', 80: '#0099ff',
  90: '#2ba8ff', 100: '#52b7ff', 110: '#78c6ff', 120: '#9ed5ff',
  130: '#c3e4ff', 140: '#e0f1ff', 150: '#f0f8ff', 160: '#f8fcff'
};

const light = createLightTheme(brand);
const dark = createDarkTheme(brand);

export function systemScheme(): Scheme {
  return matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(scheme: Scheme) {
  document.documentElement.dataset.theme = scheme;
  setTheme(scheme === 'dark' ? dark : light);
  try {
    getCurrentWindow().setTheme(scheme).catch(() => {});
  } catch {
    // plain-browser dev mode: no native window
  }
}