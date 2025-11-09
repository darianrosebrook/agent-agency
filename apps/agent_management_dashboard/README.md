# Agent Management Dashboard

A Next.js 15 dashboard application for managing agents and projects.

## Getting Started

### Prerequisites

- Node.js 20+ 
- npm or yarn

### Installation

```bash
npm install --legacy-peer-deps
```

Note: `--legacy-peer-deps` is required due to some peer dependency conflicts with React 19.

### Development

```bash
npm run dev
```

The application will be available at [http://localhost:3000](http://localhost:3000).

### Build

```bash
npm run build
```

### Production

```bash
npm run start
```

## Project Structure

```
src/
├── app/                    # Next.js App Router
│   ├── layout.tsx         # Root layout
│   ├── page.tsx           # Dashboard home page
│   ├── projects/          # Projects route
│   ├── chat/              # Chat route
│   └── phase-planner/      # Phase planner route
├── components/            # React components
│   ├── ui/                # UI component library (shadcn/ui)
│   └── ...                # Feature components
├── styles/                # Global styles
└── imports/              # Generated/imported components
```

## Routes

- `/` - Dashboard overview
- `/projects` - Project management
- `/chat` - Chat interface
- `/phase-planner` - Phase planning tool

## Technology Stack

- **Framework**: Next.js 15
- **React**: 19
- **Styling**: Tailwind CSS
- **UI Components**: Radix UI + shadcn/ui
- **Icons**: Lucide React
- **Charts**: Recharts

## Migration Notes

This project was migrated from Vite + React Router to Next.js 15 App Router. Key changes:

- React Router replaced with Next.js file-based routing
- Client components marked with `'use client'` directive
- Vite config replaced with Next.js config
- Import paths updated (removed version numbers from package imports)
- CSS updated with Tailwind directives

## License

Private project
