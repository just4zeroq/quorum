'use client'

import { ConnectButton } from '@rainbow-me/rainbowkit'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useRouter, usePathname } from 'next/navigation'
import { useTranslations } from 'next-intl'

export function Header() {
  const t = useTranslations('header')
  const router = useRouter()
  const pathname = usePathname()

  const locale = pathname.split('/')[1] || 'en'
  const otherLocale = locale === 'en' ? 'zh' : 'en'

  const switchLocale = () => {
    const newPath = pathname.replace(`/${locale}`, `/${otherLocale}`)
    router.push(newPath)
  }

  return (
    <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container flex h-16 items-center justify-between px-4">
        <div className="flex items-center gap-6">
          <a href={`/${locale}`} className="text-xl font-bold">
            {t('appName', { ns: 'common' })}
          </a>
          <nav className="flex gap-4">
            <a
              href={`/${locale}`}
              className="text-sm font-medium transition-colors hover:text-primary"
            >
              {t('markets')}
            </a>
            <a
              href={`/${locale}/portfolio`}
              className="text-sm font-medium transition-colors hover:text-primary"
            >
              {t('portfolio')}
            </a>
          </nav>
        </div>

        <div className="flex items-center gap-4">
          <Button variant="ghost" size="sm" onClick={switchLocale}>
            {otherLocale === 'en' ? 'EN' : '中文'}
          </Button>

          <ConnectButton
            chainStatus="icon"
            accountStatus="avatar"
            showBalance={false}
          />
        </div>
      </div>
    </header>
  )
}
