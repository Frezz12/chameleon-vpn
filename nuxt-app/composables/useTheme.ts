import { computed, ref } from 'vue'

const STORAGE_KEY = 'vpn_theme'
const theme = ref<'dark' | 'light'>('dark')
let initialized = false

function applyTheme(nextTheme: 'dark' | 'light') {
  if (import.meta.client) {
    document.documentElement.classList.toggle('light', nextTheme === 'light')
    localStorage.setItem(STORAGE_KEY, nextTheme)
  }
  theme.value = nextTheme
}

function initTheme() {
  if (initialized) return
  initialized = true

  if (!import.meta.client) return
  const savedTheme = localStorage.getItem(STORAGE_KEY)
  const nextTheme = savedTheme === 'light' ? 'light' : 'dark'
  applyTheme(nextTheme)
}

function setTheme(nextTheme: 'dark' | 'light') {
  applyTheme(nextTheme)
}

function toggleTheme() {
  applyTheme(theme.value === 'light' ? 'dark' : 'light')
}

export function useTheme() {
  initTheme()

  return {
    theme,
    isLight: computed(() => theme.value === 'light'),
    setTheme,
    toggleTheme,
    initTheme,
  }
}