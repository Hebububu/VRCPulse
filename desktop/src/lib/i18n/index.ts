import { writable, get } from 'svelte/store';
import en from './en';
import ko from './ko';
import jp from './jp';

type TranslationKey = keyof typeof en;

const translations: Record<string, Record<string, string>> = { en, ko, jp };

const localeOrder = ['en', 'ko', 'jp'] as const;

function detectLocale(): string {
  if (typeof window === 'undefined') return 'en';
  const saved = localStorage.getItem('vrcpulse-locale');
  if (saved && translations[saved]) return saved;
  const browserLang = navigator.language.slice(0, 2);
  if (translations[browserLang]) return browserLang;
  return 'en';
}

export const locale = writable(detectLocale());

export function t(key: TranslationKey): string {
  const current = get(locale);
  return translations[current]?.[key] ?? translations['en'][key] ?? key;
}

export function tDynamic(key: string): string {
  const current = get(locale);
  return translations[current]?.[key] ?? translations['en']?.[key] ?? key;
}

export function getLocale(): string {
  return get(locale);
}

export function setLocale(l: string) {
  if (translations[l]) {
    locale.set(l);
    localStorage.setItem('vrcpulse-locale', l);
    // Force page reload for full i18n update
    window.location.reload();
  }
}

export function toggleLocale() {
  const current = getLocale();
  const idx = localeOrder.indexOf(current as typeof localeOrder[number]);
  const next = localeOrder[(idx + 1) % localeOrder.length];
  setLocale(next);
}
