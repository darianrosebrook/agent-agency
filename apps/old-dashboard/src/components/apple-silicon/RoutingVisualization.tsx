/**
 * RoutingVisualization Component
 * Interactive model routing and load balancing visualization
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo, useRef } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import {
  Router,
  Network,
  RefreshCw,
  ArrowRight,
  Cpu,
  Zap,
  Brain,
  TrendingUp,
  TrendingDown,
  Activity,
  Clock,
  Target,
  BarChart3,
  Play,
  Pause,
  RotateCcw
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { useAppleSiliconWebSocket, useRealTimeModelMonitoring } from '@/hooks/useAppleSiliconWebSocket';
import { useAppleSiliconStore, useAppleSiliconActions } from '@/stores/apple-silicon';
import styles from './RoutingVisualization.module.scss';

// Routing data interfaces
interface RoutingNode {
  id: string;
  type: 'request' | 'ane' | 'gpu' | 'cpu';
  name: string;
  utilization: number;
  activeRequests: number;
  totalRequests: number;
  avgLatency: number;
  status: 'active' | 'idle' | 'overloaded';
}

interface RoutingFlow {
  id: string;
  from: string;
  to: string;
  requests: number;
  avgLatency: number;
  efficiency: number;
  active: boolean;
}

interface RoutingStats {
  totalRequests: number;
  routedToANE: number;
  routedToGPU: number;
  routedToCPU: number;
  avgRoutingLatency: number;
  routingEfficiency: number;
  loadBalanceScore: number;
}

export function RoutingVisualization() {
  // State management
  const [routingNodes, setRoutingNodes] = useState<RoutingNode[]>([]);
  const [routingFlows, setRoutingFlows] = useState<RoutingFlow[]>([]);
  const [routingStats, setRoutingStats] = useState<RoutingStats | null>(null);
  const [viewMode, setViewMode] = useState<'flow' | 'stats' | 'history'>('flow');
  const [animationPlaying, setAnimationPlaying] = useState(false);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // Real-time data hooks
  const { isConnected } = useAppleSiliconWebSocket();
  const { models } = useRealTimeModelMonitoring();

  // Refs for animation
  const animationFrameRef = useRef<number>();
  const animationStartTimeRef = useRef<number>();

  // Fetch routing data
  const fetchRoutingData = async () => {
    try {
      setRefreshing(true);

      // Get routing stats
      const stats = await appleSiliconApiClient.getRoutingStats();
      setRoutingStats(stats);

      // Get recent routing decisions
      const decisions = await appleSiliconApiClient.getRoutingDecisions(10);

      // Mock routing nodes (would come from API)
      const mockNodes: RoutingNode[] = [
        {
          id: 'requests',
          type: 'request',
          name: 'Incoming Requests',
          utilization: 100,
          activeRequests: stats.totalRequests,
          totalRequests: stats.totalRequests,
          avgLatency: 0,
          status: 'active'
        },
        {
          id: 'ane',
          type: 'ane',
          name: 'Apple Neural Engine',
          utilization: 78.5,
          activeRequests: Math.floor(stats.routedToANE * 0.1),
          totalRequests: stats.routedToANE,
          avgLatency: 2.3,
          status: 'active'
        },
        {
          id: 'gpu',
          type: 'gpu',
          name: 'Metal GPU',
          utilization: 45.2,
          activeRequests: Math.floor(stats.routedToGPU * 0.1),
          totalRequests: stats.routedToGPU,
          avgLatency: 8.7,
          status: 'active'
        },
        {
          id: 'cpu',
          type: 'cpu',
          name: 'CPU Cores',
          utilization: 32.1,
          activeRequests: Math.floor(stats.routedToCPU * 0.1),
          totalRequests: stats.routedToCPU,
          avgLatency: 15.2,
          status: 'active'
        }
      ];

      setRoutingNodes(mockNodes);

      // Create routing flows based on stats
      const mockFlows: RoutingFlow[] = [
        {
          id: 'req-to-ane',
          from: 'requests',
          to: 'ane',
          requests: stats.routedToANE,
          avgLatency: 2.3,
          efficiency: 0.95,
          active: true
        },
        {
          id: 'req-to-gpu',
          from: 'requests',
          to: 'gpu',
          requests: stats.routedToGPU,
          avgLatency: 8.7,
          efficiency: 0.88,
          active: true
        },
        {
          id: 'req-to-cpu',
          from: 'requests',
          to: 'cpu',
          requests: stats.routedToCPU,
          avgLatency: 15.2,
          efficiency: 0.72,
          active: true
        }
      ];

      setRoutingFlows(mockFlows);

    } catch (err) {
      console.error('Failed to fetch routing data:', err);
    } finally {
      setRefreshing(false);
    }
  };

  // Animation loop for flow visualization
  const animateFlows = () => {
    if (!animationPlaying) return;

    const currentTime = Date.now();
    const elapsed = currentTime - (animationStartTimeRef.current || currentTime);

    // Update flow animations based on elapsed time
    // This would typically update CSS custom properties or state for animations

    animationFrameRef.current = requestAnimationFrame(animateFlows);
  };

  // Handle animation controls
  const handlePlayPause = () => {
    if (animationPlaying) {
      setAnimationPlaying(false);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    } else {
      setAnimationPlaying(true);
      animationStartTimeRef.current = Date.now();
      animateFlows();
    }
  };

  const handleReset = () => {
    setAnimationPlaying(false);
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }
    // Reset animation state
  };

  // Handle node selection
  const handleNodeSelect = (nodeId: string) => {
    setSelectedNode(selectedNode === nodeId ? null : nodeId);
  };

  // Handle refresh
  const handleRefresh = async () => {
    await fetchRoutingData();
  };

  // Calculate flow strength for visualization
  const getFlowStrength = (requests: number, maxRequests: number) => {
    return Math.max(1, Math.min(5, (requests / maxRequests) * 5));
  };

  // Get hardware color
  const getHardwareColor = (type: string) => {
    switch (type) {
      case 'ane': return 'var(--color-ane)';
      case 'gpu': return 'var(--color-gpu)';
      case 'cpu': return 'var(--color-cpu)';
      case 'request': return 'var(--color-primary)';
      default: return 'var(--color-text-secondary)';
    }
  };

  // Get hardware icon
  const getHardwareIcon = (type: string) => {
    switch (type) {
      case 'ane': return <Zap size={20} />;
      case 'gpu': return <Cpu size={20} />;
      case 'cpu': return <Brain size={20} />;
      case 'request': return <Network size={20} />;
      default: return <Activity size={20} />;
    }
  };

  // Calculate max requests for flow strength
  const maxRequests = useMemo(() => {
    return Math.max(...routingFlows.map(flow => flow.requests), 1);
  }, [routingFlows]);

  // Initial data load
  useEffect(() => {
    fetchRoutingData();
  }, []);

  // Cleanup animation on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  // View mode options
  const viewModeOptions = [
    { value: 'flow', label: 'Flow Diagram', icon: Router },
    { value: 'stats', label: 'Statistics', icon: BarChart3 },
    { value: 'history', label: 'History', icon: Clock },
  ];

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Routing & Load Balancing</Text>
          <Text variant="paragraph-small" color="secondary">
            Visualize model routing decisions and load distribution
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <Activity size={12} className={styles.connected} />
            ) : (
              <Clock size={12} className={styles.disconnected} />
            )}
          </div>

          {/* Animation Controls */}
          <div className={styles.animationControls}>
            <Button
              variant="secondary"
              size="sm"
              onClick={handlePlayPause}
            >
              {animationPlaying ? <Pause size={16} /> : <Play size={16} />}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleReset}
            >
              <RotateCcw size={16} />
            </Button>
          </div>

          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
          >
            <RefreshCw
              size={16}
              className={refreshing ? styles.spinning : ''}
            />
            Refresh
          </Button>
        </div>
      </div>

      {/* View Mode Tabs */}
      <div className={styles.viewTabs}>
        {viewModeOptions.map((option) => (
          <button
            key={option.value}
            onClick={() => setViewMode(option.value as any)}
            className={`${styles.viewTab} ${viewMode === option.value ? styles.active : ''}`}
          >
            <option.icon size={16} />
            <span>{option.label}</span>
          </button>
        ))}
      </div>

      {/* Flow Diagram View */}
      {viewMode === 'flow' && (
        <div className={styles.flowDiagram}>
          <div className={styles.flowCanvas}>
            {/* Routing Nodes */}
            <div className={styles.nodesContainer}>
              {routingNodes.map((node) => (
                <div
                  key={node.id}
                  className={`${styles.routingNode} ${selectedNode === node.id ? styles.selected : ''}`}
                  onClick={() => handleNodeSelect(node.id)}
                  style={{
                    backgroundColor: getHardwareColor(node.type),
                    borderColor: selectedNode === node.id ? 'var(--color-primary)' : 'transparent'
                  }}
                >
                  <div className={styles.nodeIcon}>
                    {getHardwareIcon(node.type)}
                  </div>

                  <div className={styles.nodeContent}>
                    <Text variant="paragraph-medium" className={styles.nodeName}>
                      {node.name}
                    </Text>

                    <div className={styles.nodeMetrics}>
                      <div className={styles.nodeMetric}>
                        <Activity size={12} />
                        <span>{node.activeRequests}</span>
                      </div>

                      <div className={styles.nodeMetric}>
                        <Clock size={12} />
                        <span>{node.avgLatency.toFixed(1)}ms</span>
                      </div>
                    </div>

                    <Progress
                      value={node.utilization}
                      size="sm"
                      className={styles.nodeProgress}
                    />
                  </div>

                  <Badge
                    variant={
                      node.status === 'active' ? 'success' :
                      node.status === 'overloaded' ? 'error' : 'secondary'
                    }
                    size="sm"
                    className={styles.nodeStatus}
                  >
                    {node.status}
                  </Badge>
                </div>
              ))}
            </div>

            {/* Routing Flows */}
            <svg className={styles.flowsContainer}>
              {routingFlows.map((flow) => {
                const fromNode = routingNodes.find(n => n.id === flow.from);
                const toNode = routingNodes.find(n => n.id === flow.to);

                if (!fromNode || !toNode) return null;

                const fromIndex = routingNodes.indexOf(fromNode);
                const toIndex = routingNodes.indexOf(toNode);

                // Calculate SVG path coordinates (simplified)
                const fromX = 120 + (fromIndex * 240);
                const toX = 120 + (toIndex * 240);
                const y = 150;

                const strength = getFlowStrength(flow.requests, maxRequests);

                return (
                  <g key={flow.id} className={styles.flowGroup}>
                    {/* Flow arrow */}
                    <line
                      x1={fromX}
                      y1={y}
                      x2={toX}
                      y2={y}
                      className={styles.flowLine}
                      strokeWidth={strength}
                      style={{
                        stroke: getHardwareColor(toNode.type),
                        opacity: flow.active ? 0.8 : 0.3
                      }}
                    />

                    {/* Flow animation particles */}
                    {animationPlaying && flow.active && (
                      <circle
                        r="3"
                        fill={getHardwareColor(toNode.type)}
                        className={styles.flowParticle}
                        style={{
                          animation: `flowAnimation ${2 - (strength / 5)}s linear infinite`,
                          offsetPath: `path('M${fromX} ${y} L${toX} ${y}')`
                        }}
                      />
                    )}

                    {/* Flow stats overlay */}
                    <foreignObject
                      x={(fromX + toX) / 2 - 60}
                      y={y - 40}
                      width="120"
                      height="60"
                      className={styles.flowStats}
                    >
                      <div className={styles.flowStatsCard}>
                        <Text variant="paragraph-small" className={styles.flowRequests}>
                          {flow.requests.toLocaleString()} req
                        </Text>
                        <Text variant="paragraph-small" color="secondary">
                          {flow.avgLatency.toFixed(1)}ms avg
                        </Text>
                        <div className={styles.flowEfficiency}>
                          <span>{(flow.efficiency * 100).toFixed(0)}% efficient</span>
                        </div>
                      </div>
                    </foreignObject>
                  </g>
                );
              })}
            </svg>
          </div>

          {/* Node Details Panel */}
          {selectedNode && (
            <div className={styles.nodeDetails}>
              {(() => {
                const node = routingNodes.find(n => n.id === selectedNode);
                if (!node) return null;

                return (
                  <div className={styles.nodeDetailCard}>
                    <div className={styles.nodeDetailHeader}>
                      <div className={styles.nodeDetailIcon}>
                        {getHardwareIcon(node.type)}
                      </div>
                      <div className={styles.nodeDetailInfo}>
                        <Text variant="h4">{node.name}</Text>
                        <Badge variant="secondary">
                          {node.type.toUpperCase()}
                        </Badge>
                      </div>
                    </div>

                    <div className={styles.nodeDetailMetrics}>
                      <div className={styles.nodeDetailMetric}>
                        <Text variant="paragraph-small" color="secondary">Utilization</Text>
                        <Text variant="h3">{node.utilization.toFixed(1)}%</Text>
                        <Progress value={node.utilization} size="sm" />
                      </div>

                      <div className={styles.nodeDetailMetric}>
                        <Text variant="paragraph-small" color="secondary">Active Requests</Text>
                        <Text variant="h3">{node.activeRequests}</Text>
                      </div>

                      <div className={styles.nodeDetailMetric}>
                        <Text variant="paragraph-small" color="secondary">Total Requests</Text>
                        <Text variant="h3">{node.totalRequests.toLocaleString()}</Text>
                      </div>

                      <div className={styles.nodeDetailMetric}>
                        <Text variant="paragraph-small" color="secondary">Avg Latency</Text>
                        <Text variant="h3">{node.avgLatency.toFixed(1)}ms</Text>
                      </div>
                    </div>
                  </div>
                );
              })()}
            </div>
          )}
        </div>
      )}

      {/* Statistics View */}
      {viewMode === 'stats' && routingStats && (
        <div className={styles.statistics}>
          <div className={styles.statsGrid}>
            {/* Routing Distribution */}
            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <Target className={styles.statIcon} />
                <Text variant="h4">Routing Distribution</Text>
              </div>

              <div className={styles.routingDistribution}>
                <div className={styles.distributionItem}>
                  <div className={styles.distributionLabel}>
                    <Zap size={16} />
                    <span>ANE</span>
                  </div>
                  <div className={styles.distributionValue}>
                    <Text variant="h3">{routingStats.routedToANE.toLocaleString()}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      ({((routingStats.routedToANE / routingStats.totalRequests) * 100).toFixed(1)}%)
                    </Text>
                  </div>
                  <Progress
                    value={(routingStats.routedToANE / routingStats.totalRequests) * 100}
                    className={styles.distributionProgress}
                  />
                </div>

                <div className={styles.distributionItem}>
                  <div className={styles.distributionLabel}>
                    <Cpu size={16} />
                    <span>GPU</span>
                  </div>
                  <div className={styles.distributionValue}>
                    <Text variant="h3">{routingStats.routedToGPU.toLocaleString()}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      ({((routingStats.routedToGPU / routingStats.totalRequests) * 100).toFixed(1)}%)
                    </Text>
                  </div>
                  <Progress
                    value={(routingStats.routedToGPU / routingStats.totalRequests) * 100}
                    className={styles.distributionProgress}
                  />
                </div>

                <div className={styles.distributionItem}>
                  <div className={styles.distributionLabel}>
                    <Brain size={16} />
                    <span>CPU</span>
                  </div>
                  <div className={styles.distributionValue}>
                    <Text variant="h3">{routingStats.routedToCPU.toLocaleString()}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      ({((routingStats.routedToCPU / routingStats.totalRequests) * 100).toFixed(1)}%)
                    </Text>
                  </div>
                  <Progress
                    value={(routingStats.routedToCPU / routingStats.totalRequests) * 100}
                    className={styles.distributionProgress}
                  />
                </div>
              </div>
            </div>

            {/* Performance Metrics */}
            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <TrendingUp className={styles.statIcon} />
                <Text variant="h4">Performance Metrics</Text>
              </div>

              <div className={styles.performanceMetrics}>
                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Avg Routing Latency</Text>
                  <Text variant="h3">{routingStats.avgRoutingLatency.toFixed(2)}ms</Text>
                </div>

                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Routing Efficiency</Text>
                  <Text variant="h3">{(routingStats.routingEfficiency * 100).toFixed(1)}%</Text>
                </div>

                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Load Balance Score</Text>
                  <Text variant="h3">{routingStats.loadBalanceScore.toFixed(1)}/10</Text>
                </div>

                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Total Requests</Text>
                  <Text variant="h3">{routingStats.totalRequests.toLocaleString()}</Text>
                </div>
              </div>
            </div>

            {/* Load Balancing Visualization */}
            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <BarChart3 className={styles.statIcon} />
                <Text variant="h4">Load Balancing</Text>
              </div>

              <div className={styles.loadBalanceViz}>
                {routingNodes.filter(node => node.type !== 'request').map((node) => (
                  <div key={node.id} className={styles.loadBalanceItem}>
                    <div className={styles.loadBalanceHeader}>
                      {getHardwareIcon(node.type)}
                      <Text variant="paragraph-medium">{node.name}</Text>
                      <Badge variant="secondary" size="sm">
                        {node.utilization.toFixed(1)}%
                      </Badge>
                    </div>

                    <Progress
                      value={node.utilization}
                      className={styles.loadBalanceProgress}
                      variant={
                        node.utilization > 80 ? 'warning' :
                        node.utilization > 60 ? 'secondary' : 'success'
                      }
                    />

                    <div className={styles.loadBalanceStats}>
                      <Text variant="paragraph-small" color="secondary">
                        {node.activeRequests} active • {node.totalRequests.toLocaleString()} total
                      </Text>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* History View */}
      {viewMode === 'history' && (
        <div className={styles.history}>
          <div className={styles.historyCard}>
            <div className={styles.historyHeader}>
              <Text variant="h4">Routing Decision History</Text>
              <Text variant="paragraph-small" color="secondary">
                Recent routing decisions and their outcomes
              </Text>
            </div>

            <div className={styles.historyPlaceholder}>
              <Clock size={48} className={styles.historyIcon} />
              <Text variant="h5">History Analysis Coming Soon</Text>
              <Text variant="paragraph-medium" color="secondary" className={styles.historyText}>
                Detailed routing decision history with trend analysis,
                performance comparisons, and optimization insights
                will be available in the next update.
              </Text>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
