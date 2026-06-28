import { ref, computed } from 'vue'
import en from '~/locales/en'
import ru from '~/locales/ru'

type LocaleDict = Record<string, string>

const locales: Record<string, LocaleDict> = { en, ru }

const STORAGE_KEY = 'vpn_locale'

function getInitialLocale(): string {
  if (import.meta.client) {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'ru' || saved === 'en') return saved
    const lang = navigator.language?.startsWith('ru') ? 'ru' : 'en'
    return lang
  }
  return 'en'
}

const currentLocale = ref<string>(getInitialLocale())
const currentDict = computed<LocaleDict>(() => locales[currentLocale.value] || en)

function t(key: string, fallback?: string): string {
  return currentDict.value[key] || fallback || key
}

function setLocale(locale: string) {
  if (locale !== 'en' && locale !== 'ru') return
  currentLocale.value = locale
  if (import.meta.client) {
    localStorage.setItem(STORAGE_KEY, locale)
  }
}

function toggleLocale() {
  setLocale(currentLocale.value === 'en' ? 'ru' : 'en')
}

export function useI18n() {
  return {
    t,
    locale: currentLocale,
    setLocale,
    toggleLocale,
    isRu: computed(() => currentLocale.value === 'ru'),
  }
}
