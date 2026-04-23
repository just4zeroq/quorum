'use client'

import { useTranslations } from 'next-intl'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'

interface Position {
  id: string
  market: string
  outcome: string
  shares: number
  avgPrice: number
  currentPrice: number
  pnl: number
}

interface Order {
  id: string
  market: string
  outcome: string
  side: 'buy' | 'sell'
  price: number
  shares: number
  status: 'pending' | 'filled' | 'cancelled'
  timestamp: string
}

const mockPositions: Position[] = [
  {
    id: '1',
    market: 'Will Bitcoin exceed $100,000 by end of 2025?',
    outcome: 'Yes',
    shares: 100,
    avgPrice: 0.55,
    currentPrice: 0.65,
    pnl: 15.38,
  },
]

const mockOrders: Order[] = [
  {
    id: '1',
    market: 'Will ETH flip BTC?',
    outcome: 'Yes',
    side: 'buy',
    price: 0.15,
    shares: 50,
    status: 'filled',
    timestamp: '2024-10-15 10:30',
  },
]

export default function PortfolioPage() {
  const t = useTranslations('portfolio')
  const tTrading = useTranslations('trading')

  const formatPrice = (price: number) => `${(price * 100).toFixed(1)}%`

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold tracking-tight">{t('title')}</h1>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total Value
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">$1,234.56</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              {t('unrealizedPnl')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">+$123.45</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              {t('realizedPnl')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">+$456.78</div>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="positions" className="space-y-4">
        <TabsList>
          <TabsTrigger value="positions">{t('positions')}</TabsTrigger>
          <TabsTrigger value="pending">{t('pendingOrders')}</TabsTrigger>
          <TabsTrigger value="history">{t('history')}</TabsTrigger>
        </TabsList>

        <TabsContent value="positions">
          <Card>
            <CardContent className="p-0">
              {mockPositions.length === 0 ? (
                <div className="p-6 text-center text-muted-foreground">
                  {t('noPositions')}
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Market</TableHead>
                      <TableHead>Outcome</TableHead>
                      <TableHead>Shares</TableHead>
                      <TableHead>Avg Price</TableHead>
                      <TableHead>Current</TableHead>
                      <TableHead>{t('pnl')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {mockPositions.map((position) => (
                      <TableRow key={position.id}>
                        <TableCell className="font-medium max-w-[200px] truncate">
                          {position.market}
                        </TableCell>
                        <TableCell>{position.outcome}</TableCell>
                        <TableCell>{position.shares}</TableCell>
                        <TableCell>{formatPrice(position.avgPrice)}</TableCell>
                        <TableCell>{formatPrice(position.currentPrice)}</TableCell>
                        <TableCell className={position.pnl >= 0 ? 'text-green-500' : 'text-red-500'}>
                          {position.pnl >= 0 ? '+' : ''}{position.pnl.toFixed(2)}%
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="pending">
          <Card>
            <CardContent className="p-0">
              {mockOrders.filter(o => o.status === 'pending').length === 0 ? (
                <div className="p-6 text-center text-muted-foreground">
                  {t('noPendingOrders')}
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Market</TableHead>
                      <TableHead>Side</TableHead>
                      <TableHead>Price</TableHead>
                      <TableHead>Shares</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {mockOrders
                      .filter(o => o.status === 'pending')
                      .map((order) => (
                        <TableRow key={order.id}>
                          <TableCell className="font-medium max-w-[200px] truncate">
                            {order.market}
                          </TableCell>
                          <TableCell>
                            <Badge variant={order.side === 'buy' ? 'default' : 'secondary'}>
                              {order.side.toUpperCase()}
                            </Badge>
                          </TableCell>
                          <TableCell>{formatPrice(order.price)}</TableCell>
                          <TableCell>{order.shares}</TableCell>
                          <TableCell>
                            <Badge variant="outline">{order.status}</Badge>
                          </TableCell>
                          <TableCell>
                            <Button variant="ghost" size="sm">
                              Cancel
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="history">
          <Card>
            <CardContent className="p-0">
              {mockOrders.length === 0 ? (
                <div className="p-6 text-center text-muted-foreground">
                  {t('noHistory')}
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Market</TableHead>
                      <TableHead>Side</TableHead>
                      <TableHead>Price</TableHead>
                      <TableHead>Shares</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead>Time</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {mockOrders.map((order) => (
                      <TableRow key={order.id}>
                        <TableCell className="font-medium max-w-[200px] truncate">
                          {order.market}
                        </TableCell>
                        <TableCell>
                          <Badge variant={order.side === 'buy' ? 'default' : 'secondary'}>
                            {order.side.toUpperCase()}
                          </Badge>
                        </TableCell>
                        <TableCell>{formatPrice(order.price)}</TableCell>
                        <TableCell>{order.shares}</TableCell>
                        <TableCell>
                          <Badge variant="outline">{order.status}</Badge>
                        </TableCell>
                        <TableCell className="text-muted-foreground">
                          {order.timestamp}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
