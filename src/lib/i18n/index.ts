import { register, init, getLocaleFromNavigator, locale } from 'svelte-i18n';
import zh from './zh.json';
import en from './en.json';

register('zh', () => Promise.resolve(zh));
register('en', () => Promise.resolve(en));

export function resolveLocale(pref: 'system' | 'zh' | 'en'): 'zh' | 'en' {
  if (pref !== 'system') return pref;
  const nav = getLocaleFromNavigator() ?? 'zh';
  return nav.toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

export function setupI18n(pref: 'system' | 'zh' | 'en') {
  init({ fallbackLocale: 'zh', initialLocale: resolveLocale(pref) });
}

export function setLocale(pref: 'system' | 'zh' | 'en') {
  locale.set(resolveLocale(pref));
}

export { locale };