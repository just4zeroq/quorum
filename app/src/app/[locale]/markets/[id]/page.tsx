'use client'

import { useTranslations } from 'next-intl'
import { useParams, useRouter } from 'next/navigation'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

interface Outcome {
  id: string
  name: string
  price: number
  volume: number
}

interface Market {
  id: string
  question: string
  description: string
  outcomes: Outcome[]
  totalVolume: number
  liquidity: number
  endDate: string
  status: 'trading' | 'resolved' | 'closed'
  winner?: string
}

const mockMarket: Market = {
  id: '1',
  question: 'Will Bitcoin exceed $100,000 by end of 2025?',
  description:
    'This market resolves to YES if the price of Bitcoin exceeds $100,000 USD on any major exchange before December 31, 2025 11:59 PM UTC. The price will be calculated using the average price across major exchanges including Binance, Coinbase, and Kraken.',
  outcomes: [
    { id: 'yes', name: 'Yes', price: 0.65, volume: 1200000 },
    { id: 'no', name: 'No', price: 0.35, volume: 800000 },
  ],
  totalVolume: 2000000,
  liquidity: 500000,
  endDate: '2025-12-31',
  status: 'trading',
}

export default function MarketDetailPage() {
  const t = useTranslations('marketDetail')
  const tTrading = useTranslations('trading')
  const tCommon = useTranslations('common')
  const params = useParams()
  const router = useRouter()
  const locale = params.locale as string

  const market = mockMarket
  const isResolved = market.status === 'resolved'

  const formatPrice = (price: number) => `${(price * 100).toFixed(1)}%`
  const formatVolume = (volume: number) => {
    if (volume >= 1000000) return `$${(volume / 1000000).toFixed(1)}M`
    if (volume >= 1000) return `$${(volume / 1000).toFixed(0)}K`
    return `$${volume}`
  }

  return (
    <div className="space-y-6">
      <Button
        variant="ghost"
        onClick={() => router.push(`/${locale}`)}
        className="mb-4"
      >
        ← {tCommon('back')}
      </Button>

      <div className="grid gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2 space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-start justify-between">
                <CardTitle className="text-2xl">{market.question}</CardTitle>
                {isResolved && (
                  <Badge variant="secondary">
                    {t('winner')}: {market.winner}
                  </Badge>
                )}
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-muted-foreground">{market.description}</p>
              <div className="flex gap-4 text-sm text-muted-foreground">
                <span>
                  {t('volume24h')}: {formatVolume(market.totalVolume)}
                </span>
                <span>
                  {t('liquidity')}: {formatVolume(market.liquidity)}
                </span>
                <span>
                  {t('resolutionDate')}: {market.endDate}
                </span>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>{t('outcomes')}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {market.outcomes.map((outcome) => (
                  <div
                    key={outcome.id}
                    className="flex items-center justify-between p-4 rounded-lg border"
                  >
                    <div>
                      <div className="font-medium">{outcome.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {formatPrice(outcome.price)} · {formatVolume(outcome.volume)}{' '}
                        {t('totalVolume').toLowerCase()}
                      </div>
                    </div>
                    {!isResolved && (
                      <div className="flex items-center gap-2">
                        <Button variant="outline" size="sm">
                          {tTrading('sell')}
                        </Button>
                        <Button size="sm">{tTrading('buy')}</Button>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>{tTrading('placeOrder')}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>{tTrading('shares')}</Label>
                <Input type="number" placeholder="0.00" />
              </div>

              <div className="space-y-2">
                <Label>{tTrading('price')}</Label>
                <div className="flex items-center gap-2">
                  <Input type="number" placeholder="0.00" disabled />
                  <span className="text-sm text-muted-foreground">USDC</span>
                </div>
              </div>

              <Separator />

              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{tTrading('total')}</span>
                  <span>0.00 USDC</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    {tTrading('potentialPayout')}
                  </span>
                  <span>0.00 USDC</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    {tTrading('probability')}
                  </span>
                  <span>--</span>
                </div>
              </div>

              <Button className="w-full">{tTrading('placeOrder')}</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Order Book</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between font-medium">
                  <span>Price</span>
                  <span>Shares</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">0.66</span>
                  <span>1,234</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">0.65</span>
                  <span>5,678</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">0.64</span>
                  <span>3,456</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
