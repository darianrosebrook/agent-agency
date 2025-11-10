# Agent Agency V3 Dashboard

Next.js 15 dashboard application for monitoring and managing the Agent Agency V3 system.

## Features

- **System Monitoring**: View system health, metrics, and analytics
- **Project Management**: Track and manage projects
- **Task Monitoring**: View task execution details with chain-of-thought visualization
- **Database Inspection**: Browse database tables and execute queries
- **Provenance Tracking**: View audit trails and provenance records

## Getting Started

### Prerequisites

- Node.js 18+ 
- Agent Agency V3 API server running

### Installation

1. Install dependencies:
```bash
npm install
```

2. Copy environment variables:
```bash
cp .env.local.example .env.local
```

3. Configure environment variables in `.env.local`:
```
NEXT_PUBLIC_API_URL=http://localhost:8080
API_ADMIN_USERNAME=admin
API_ADMIN_PASSWORD=your_password
```

4. Run the development server:
```bash
npm run dev
```

5. Open [http://localhost:3000](http://localhost:3000) in your browser.

## Project Structure

```
src/
├── app/                    # Next.js App Router pages
├── components/            # React components
│   ├── ui/                # Design system components
│   ├── layout/            # Layout components
│   └── dashboard/         # Dashboard-specific components
├── lib/                   # Utilities and services
│   ├── api/               # API client and endpoints
│   ├── hooks/             # React hooks
│   └── utils/             # Utility functions
├── styles/                # SCSS styles
│   ├── tokens/            # Design tokens
│   └── base/              # Base styles
└── types/                 # TypeScript types
```

## Design System

The dashboard uses a custom design system built with SCSS modules:

- **Design Tokens**: Colors, typography, spacing, breakpoints
- **Components**: Reusable UI components (Button, Card, Table, Badge, etc.)
- **Layout**: Sidebar navigation and header layout

## API Integration

The dashboard connects to the Agent Agency V3 observational API:

- All API calls are proxied through Next.js API routes for security
- Admin authentication is handled server-side
- JWT tokens are managed automatically

## Development

```bash
# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start

# Run linting
npm run lint
```

## Technologies

- **Next.js 15**: React framework with App Router
- **TypeScript**: Type safety
- **SCSS Modules**: Component-scoped styling
- **Axios**: HTTP client
- **SWR**: Data fetching and caching (optional)
- **date-fns**: Date formatting utilities
