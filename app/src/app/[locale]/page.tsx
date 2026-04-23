'use client'

import { useTranslations } from 'next-intl'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { useParams, useRouter } from 'next/navigation'

interface Market {
  id: string
  question: string
  description: string
  outcomes: { id: string; name: string; price: number; volume: number }[]
  totalVolume: number
  endDate: string
  status: 'trading' | 'resolved' | 'closed'
  winner?: string
}

const mockMarkets: Market[] = [
  {
    id: '1',
    question: 'Will Bitcoin exceed $100,000 by end of 2025?',
    description: 'This market resolves to YES if the price of Bitcoin exceeds $100,000 USD on any major exchange before December 31, 2025 11:59 PM UTC.',
    outcomes: [
      { id: 'yes', name: 'Yes', price: 0.65, volume: 1200000 },
      { id: 'no', name: 'No', price: 0.35, volume: 800000 },
    ],
    totalVolume: 2000000,
    endDate: '2025-12-31',
    status: 'trading',
  },
  {
    id: '2',
    question: 'Will Ethereum flip Bitcoin market cap in 2025?',
    description: 'This market resolves based on whether Ethereum total market cap exceeds Bitcoin market cap at any point during 2025.',
    outcomes: [
      { id: 'yes', name: 'Yes', price: 0.15, volume: 300000 },
      { id: 'no', name: 'No', price: 0.85, volume: 1500000 },
    ],
    totalVolume: 1800000,
    endDate: '2025-12-31',
    status: 'trading',
  },
  {
    id: '3',
    question: 'Who will win the 2024 US Presidential Election?',
    description: 'This market resolves based on the winner of the 2024 US Presidential Election.',
    outcomes: [
      { id: 'trump', name: 'Trump', price: 0.52, volume: 5000000 },
      { id: 'harris', name: 'Harris', price: 0.48, volume: 4500000 },
    ],
    totalVolume: 9500000,
    endDate: '2024-11-05',
    status: 'resolved',
    winner: 'trump',
  },
]

export default function MarketsPage() {
  const t = useTranslations('markets')
  const tTrading = useTranslations('trading')
  const params = useParams()
  const router = useRouter()
  const locale = params.locale as string

  const getStatusBadge = (status: Market['status']) => {
    switch (status) {
      case 'trading':
        return <Badge variant="default">{t('trading')}</Badge>
      case 'resolved':
        return <Badge variant="secondary">{t('resolved')}</Badge>
      case 'closed':
        return <Badge variant="outline">{t('closed')}</Badge>
    }
  }

  const formatPrice = (price: number) => `${(price * 100).toFixed(1)}%`

  const formatVolume = (volume: number) => {
    if (volume >= 1000000) return `$${(volume / 1000000).toFixed(1)}M`
    if (volume >= 1000) return `$${(volume / 1000).toFixed(0)}K`
    return `$${volume}`
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">{t('title')}</h1>
      </div>

      <div className="flex gap-4">
        <Input placeholder={t('search')} className="max-w-sm" />
      </div>

      <Tabs defaultValue="all" className="space-y-4">
        <TabsList>
          <TabsTrigger value="all">All</TabsTrigger>
          <TabsTrigger value="trading">{t('trading')}</TabsTrigger>
          <TabsTrigger value="resolved">{t('resolved')}</TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {mockMarkets.map((market) => (
              <Card
                key={market.id}
                className="cursor-pointer transition-colors hover:bg-muted/50"
                onClick={() => router.push(`/${locale}/markets/${market.id}`)}
              >
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <CardTitle className="text-lg font-medium">
                      {market.question}
                    </CardTitle>
                    {getStatusBadge(market.status)}
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <p className="text-sm text-muted-foreground line-clamp-2">
                    {market.description}
                  </p>

                  <Separator />

                  <div className="space-y-2">
                    {market.outcomes.map((outcome) => (
                      <div
                        key={outcome.id}
                        className="flex items-center justify-between"
                      >
                        <span className="text-sm">{outcome.name}</span>
                        <div className="flex items-center gap-4">
                          <span className="text-sm font-medium">
                            {formatPrice(outcome.price)}
                          </span>
                          <span className="text-xs text-muted-foreground">
                            {formatVolume(outcome.volume)}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>

                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>
                      {t('volume')}: {formatVolume(market.totalVolume)}
                    </span>
                    <span>
                      {t('endDate')}: {market.endDate}
                    </span>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
