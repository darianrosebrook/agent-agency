import type { Metadata, Viewport } from 'next'
import '@/styles/globals.scss'

export const metadata: Metadata = {
  title: 'Agent Agency V3 Dashboard',
  description: 'Real-time monitoring and conversational interface for Agent Agency V3',
  keywords: ['agent', 'monitoring', 'dashboard', 'real-time', 'observability'],
  authors: [{ name: '@darianrosebrook' }],
  viewport: 'width=device-width, initial-scale=1',
}

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="dashboard-body">
        {children}
      </body>
    </html>
  )
}
