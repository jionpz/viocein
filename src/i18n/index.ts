import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from './locales/en.json'
import zh from './locales/zh.json'

// Internal build ships English + Simplified Chinese only. Anything else
// (including a `ui_language` left over from a downgraded install) falls back to
// English instead of leaving `i18n.language` on an unsupported code.
export const SUPPORTED_LANGUAGES = ['en', 'zh'] as const

const savedLang = (() => {
  if (typeof localStorage === 'undefined') return 'en'
  const saved = localStorage.getItem('ui_language')
  return SUPPORTED_LANGUAGES.includes(saved as (typeof SUPPORTED_LANGUAGES)[number])
    ? (saved as string)
    : 'en'
})()

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    zh: { translation: zh },
  },
  lng: savedLang,
  fallbackLng: 'en',
  supportedLngs: [...SUPPORTED_LANGUAGES],
  interpolation: { escapeValue: false },
})

export default i18n
