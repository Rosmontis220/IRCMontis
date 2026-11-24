import { getCurrentWindow } from '@tauri-apps/api/window';

export type Scheme = 'light' | 'dark';

export function systemScheme(): Scheme {
  return matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

// forceNative=false leaves the WebView2 media query following the OS, which is
// required for "system" mode: forcing a native theme also overrides
// prefers-color-scheme, so systemScheme() would read the forced value instead.
export function applyTheme(scheme: Scheme, forceNative = true) {
  document.documentElement.dataset.theme = scheme;
  if (forceNative) {
    try {
      getCurrentWindow().setTheme(scheme).catch(() => {});
    } catch {
      // plain-browser dev mode: no native window
    }
  }
}

export async function applySystemTheme() {
  try {
    await getCurrentWindow().setTheme(null);
  } catch {
    // plain-browser dev mode: no native window
  }
  applyTheme(systemScheme(), false);
}