import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from './en'
import zh from './zh'

const saved = localStorage.getItem('lang')
const lng = saved === 'zh' ? 'zh' : 'en'   // default: en

i18n.use(initReactI18next).init({
  lng,
  fallbackLng: 'en',
  interpolation: { escapeValue: false },   // React already escapes
  resources: {
    en: { translation: en },
    zh: { translation: zh },
  },
})

export default i18n
