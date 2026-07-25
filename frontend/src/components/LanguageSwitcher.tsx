import { useTranslation } from 'react-i18next'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

export default function LanguageSwitcher() {
  const { i18n, t } = useTranslation()
  return (
    <Select
      value={i18n.language}
      onValueChange={(v) => {
        i18n.changeLanguage(v)
        localStorage.setItem('lang', v)
      }}
    >
      <SelectTrigger className="w-28 h-8 text-sm">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="en">{t('langEn')}</SelectItem>
        <SelectItem value="zh">{t('langZh')}</SelectItem>
      </SelectContent>
    </Select>
  )
}
