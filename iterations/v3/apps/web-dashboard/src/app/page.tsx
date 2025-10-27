/**
 * Dashboard Page - Next.js 16 Server Component
 * 
 * @author @darianrosebrook
 * 
 * Modernized dashboard using Server Components, Suspense boundaries,
 * and FlowPress design system. Optimized for minimal CLS.
 */

'use client';

import { Suspense, useEffect, useRef, lazy, useState } from 'react';
import { gsap } from 'gsap';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import { Text } from '@/design-system/primitives';
import { useStaggerAnimation } from '@/interactions';
import styles from './page.module.scss';

// Workspace Composer
import {
  ComposerProvider,
  Composer,
  EditableInput,
  CommandPalette,
  ContextTray,
  ModeSelect,
  SendButton,
  SendTimingMenu,
  QuickSettingsMenu,
  AttachMenu,
  useComposer,
  type CommandDef,
  type MessageToken,
  type SendPayload,
} from '@/components/workspace-composer';

// Lazy load heavy components for better performance
const MetricsSection = lazy(() => import('@/components/shared/MetricsSection'));
const QuickActions = lazy(() => import('@/components/shared/QuickActions'));
const SystemStatusCard = lazy(() => import('@/components/shared/SystemStatusCard'));
const RecentTasksCard = lazy(() => import('@/components/shared/RecentTasksCard'));
const SLODashboard = lazy(() => import('@/components/monitoring/SLODashboard'));
const SLOAlertsDashboard = lazy(() => import('@/components/monitoring/SLOAlertsDashboard'));

// Workspace Composer Commands
const COMMANDS: CommandDef[] = [
  { value: "/doc", label: "New Document", description: "Create a new document" },
  { value: "/agent", label: "Agent Mode", description: "Start an agent task" },
  { value: "/plan", label: "Planning", description: "Create a plan or roadmap" },
  { value: "/code", label: "Code Block", description: "Insert a code snippet" },
  { value: "/idea", label: "Brainstorm", description: "Generate ideas" },
];

// Development utilities (tree-shaken in production)
if (process.env.NODE_ENV === 'development') {
  import('@/utils/responsive-test').then(() => {
    // Layout testing utilities available in dev mode
  });
}

/**
 * Workspace Composer Shell - Handles prompt interface when no tasks are running
 */
function WorkspaceComposerShell() {
  const { commands, contextItems, setContextItems, meta, onSend } = useComposer();

  const [tokens, setTokens] = useState<MessageToken[]>([]);
  const [canSend, setCanSend] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [paletteAnchor, setPaletteAnchor] = useState<{ top: number; left: number } | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const handleChange = (next: MessageToken[], text: string) => {
    setTokens(next);
    setCanSend(text.trim().length > 0);
  };

  const handleSlashBoundary = (rect: DOMRect) => {
    setPaletteOpen(true);
    setPaletteAnchor({ top: rect.top - 250, left: rect.left + 20 });
  };

  const handleSelectCommand = (cmd: CommandDef) => {
    setTokens((prev) => [...prev, { kind: "command", command: cmd.value }]);
    setPaletteOpen(false);
  };

  const handleSend = () => {
    const text = tokens
      .map((t) => (t.kind === "text" ? t.text : `${t.command}${t.value ? " " + t.value : ""}`))
      .join(" ")
      .trim();

    const payload: SendPayload = {
      tokens,
      text,
      meta,
      context: contextItems,
    };

    onSend?.(payload);
    setTokens([]);
  };

  return (
    <>
      <div className="text-center space-y-2 my-6">
        <Text variant="display-2" className={styles.title}>
          Start Creating
        </Text>
        <Text variant="paragraph-large" color="secondary" align="center">
          Ask me anything about your workspace…
        </Text>
      </div>

      <Composer
        ContextTray={
          <ContextTray
            items={contextItems}
            expandedId={expandedId}
            onToggleExpand={(id) => setExpandedId((x) => (x === id ? null : id))}
            onRemove={(id) => setContextItems((xs) => xs.filter((i) => i.id !== id))}
          />
        }
        InputArea={
          <EditableInput
            value={tokens}
            onChange={handleChange}
            onEnterSend={handleSend}
            onSlashBoundary={(rect) => handleSlashBoundary(rect)}
          />
        }
        FooterLeft={
          <>
            <AttachMenu onAdd={(item) => setContextItems((xs) => [...xs, item])} />
            <QuickSettingsMenu />
            <ModeSelect />
            <SendTimingMenu />
          </>
        }
        FooterRight={<SendButton disabled={!canSend} onClick={handleSend} />}
      />

      <CommandPalette
        open={paletteOpen}
        anchor={paletteAnchor}
        commands={commands}
        onSelect={handleSelectCommand}
        onClose={() => setPaletteOpen(false)}
      />
    </>
  );
}

/**
 * Skeleton for dashboard cards
 * Maintains exact dimensions to prevent CLS
 */
function CardSkeleton() {
  return (
    <div 
      className={styles.card} 
      style={{ 
        minHeight: '200px',
        height: '200px',
        maxHeight: '200px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true" />
        <span className="sr-only">Loading card content...</span>
      </div>
    </div>
  );
}

/**
 * Skeleton for metrics section
 * Reserves space for metric tiles
 */
function MetricsSkeleton() {
  return (
    <div 
      className={styles.metricsSection} 
      style={{ 
        minHeight: '400px',
        height: '400px',
        maxHeight: '400px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true" />
        <p aria-live="polite">Loading metrics...</p>
      </div>
    </div>
  );
}

/**
 * Skeleton for SLO section
 * Maintains consistent height
 */
function SLOSkeleton() {
  return (
    <div 
      className={styles.sloSection} 
      style={{ 
        minHeight: '300px',
        height: '300px',
        maxHeight: '300px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true" />
        <p aria-live="polite">Loading service level objectives...</p>
      </div>
    </div>
  );
}

/**
 * Main Dashboard Page Component
 * Uses Suspense boundaries and GSAP animations
 * Includes workspace composer for when no tasks are running
 */
export default function DashboardPage() {
  const [showComposer, setShowComposer] = useState(false);
  const headerRef = useRef<HTMLElement>(null);
  
  // Immediate fade-in animation for header (always visible at top)
  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    // Set initial state
    gsap.set(header, { opacity: 0, y: -20 });

    // Animate in immediately
    gsap.to(header, {
      opacity: 1,
      y: 0,
      duration: 0.6,
      delay: 0.1,
      ease: 'power3.out',
    });
  }, []);

  const metricsRef = useRef<HTMLElement>(null);
  const sloRef = useRef<HTMLElement>(null);

  // Immediate animations for above-the-fold sections
  useEffect(() => {
    const metrics = metricsRef.current;
    const slo = sloRef.current;

    if (metrics) {
      gsap.set(metrics, { opacity: 0, y: 30 });
      gsap.to(metrics, {
        opacity: 1,
        y: 0,
        duration: 0.6,
        delay: 0.2,
        ease: 'power3.out',
      });
    }

    if (slo) {
      gsap.set(slo, { opacity: 0, y: 30 });
      gsap.to(slo, {
        opacity: 1,
        y: 0,
        duration: 0.6,
        delay: 0.3,
        ease: 'power3.out',
      });
    }
  }, []);
  
  // Stagger animation for card grid
  const { ref: cardsGridRef } = useStaggerAnimation<HTMLDivElement>({
    delay: 0.4,
    stagger: 0.1,
    duration: 0.5,
    type: 'slideUp',
  });

  const handleComposerSend = (payload: SendPayload) => {
    // For now, just log and switch back to dashboard
    console.log('Composer send:', payload);
    // In the future, this would route to a new workspace/document
    setShowComposer(false);
  };

  // Determine if we should show composer (when no active tasks)
  // For now, we'll add a toggle button to switch views
  const toggleView = () => setShowComposer(!showComposer);

  return (
    <DashboardLayout>
      <main role="main" aria-label="Dashboard" className={styles.container}>
        {/* Page Header - Immediate fade in animation with bold typography */}
        <header ref={headerRef} className={styles.header}>
          <Text variant="display-2" align="center" className={styles.title} id="page-title">
            {showComposer ? "Start Creating" : "Dashboard"}
          </Text>
          <Text variant="paragraph-large" color="secondary" align="center" className={styles.subtitle}>
            {showComposer
              ? "Ask me anything about your workspace…"
              : "Welcome to Agent Agency V3. Monitor task execution and system health."
            }
          </Text>
          {/* Toggle between dashboard and composer */}
          <div className="flex justify-center mt-4">
            <button
              onClick={toggleView}
              className="px-4 py-2 text-sm bg-muted hover:bg-muted/80 rounded-md transition-colors"
            >
              {showComposer ? "← Back to Dashboard" : "Start Creating →"}
            </button>
          </div>
        </header> 

        {/* Connection Banner - only shown when offline */}
        <OnlineOnly>
          <ConnectionBanner />
        </OnlineOnly>

        {showComposer ? (
          /* Workspace Composer - When no tasks are running */
          <ComposerProvider commands={COMMANDS} onSend={handleComposerSend}>
            <WorkspaceComposerShell />
          </ComposerProvider>
        ) : (
          <>
            {/* Metrics Section - Above the fold for immediate visibility */}
            <section ref={metricsRef} className={styles.metricsSection} aria-labelledby="metrics-heading">
              <Suspense fallback={<MetricsSkeleton />}>
                <MetricsSection />
              </Suspense>
            </section>

            {/* Service Level Objectives Dashboard */}
            <section ref={sloRef} className={styles.sloSection} aria-labelledby="slo-heading">
              <Suspense fallback={<SLOSkeleton />}>
                <SLODashboard />
              </Suspense>
            </section>

            {/* Card Grid - Staggered animation for engaging entry */}
            <section
              ref={cardsGridRef}
              className={styles.cardsGrid}
              aria-labelledby="cards-heading"
              role="region"
            >
              <Suspense fallback={<CardSkeleton />}>
                <QuickActions />
              </Suspense>

              <Suspense fallback={<CardSkeleton />}>
                <SystemStatusCard />
              </Suspense>

              <Suspense fallback={<CardSkeleton />}>
                <RecentTasksCard />
              </Suspense>

              <Suspense fallback={<CardSkeleton />}>
                <SLOAlertsDashboard />
              </Suspense>
            </section>
          </>
        )}
      </main>
    </DashboardLayout>
  );
}

