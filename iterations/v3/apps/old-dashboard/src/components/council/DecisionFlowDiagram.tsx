/**
 * DecisionFlowDiagram Component
 * Visual representation of council decision-making process
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  GitBranch,
  Users,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Clock,
  ArrowRight,
  ZoomIn,
  ZoomOut,
  RotateCcw
} from 'lucide-react';
import styles from './DecisionFlowDiagram.module.scss';

interface DecisionNode {
  id: string;
  type: 'input' | 'judge' | 'consensus' | 'output';
  title: string;
  description: string;
  status: 'pending' | 'processing' | 'completed' | 'error';
  data?: any;
  position: { x: number; y: number };
}

interface DecisionEdge {
  id: string;
  sourceId: string;
  targetId: string;
  type: 'judge_vote' | 'consensus_calc' | 'final_decision';
  data?: any;
}

interface ActiveDecision {
  id: string;
  title: string;
  status: 'pending' | 'in_progress' | 'completed' | 'intervened';
  currentStage: string;
  progress: number;
  judges: Array<{
    id: string;
    name: string;
    status: 'pending' | 'voting' | 'completed';
    vote?: 'approve' | 'reject' | 'uncertain';
  }>;
}

export function DecisionFlowDiagram() {
  const [zoom, setZoom] = useState(1);
  const [selectedDecision, setSelectedDecision] = useState<ActiveDecision | null>(null);

  // Mock active decision
  const activeDecision: ActiveDecision = {
    id: 'decision-001',
    title: 'Content Moderation Review',
    status: 'in_progress',
    currentStage: 'Judge Deliberation',
    progress: 65,
    judges: [
      {
        id: 'judge-1',
        name: 'Ethical Judge',
        status: 'completed',
        vote: 'approve',
      },
      {
        id: 'judge-2',
        name: 'Safety Judge',
        status: 'completed',
        vote: 'reject',
      },
      {
        id: 'judge-3',
        name: 'Context Judge',
        status: 'voting',
      },
      {
        id: 'judge-4',
        name: 'Compliance Judge',
        status: 'pending',
      },
    ],
  };

  // Mock decision flow nodes
  const nodes: DecisionNode[] = [
    {
      id: 'input-1',
      type: 'input',
      title: 'Task Submission',
      description: 'Content moderation request received',
      status: 'completed',
      position: { x: 100, y: 100 },
    },
    {
      id: 'judge-1',
      type: 'judge',
      title: 'Ethical Judge',
      description: 'Evaluating ethical implications',
      status: 'completed',
      position: { x: 300, y: 50 },
    },
    {
      id: 'judge-2',
      type: 'judge',
      title: 'Safety Judge',
      description: 'Assessing safety risks',
      status: 'completed',
      position: { x: 300, y: 150 },
    },
    {
      id: 'judge-3',
      type: 'judge',
      title: 'Context Judge',
      description: 'Analyzing context and intent',
      status: 'processing',
      position: { x: 300, y: 250 },
    },
    {
      id: 'consensus-1',
      type: 'consensus',
      title: 'Consensus Engine',
      description: 'Calculating final decision',
      status: 'pending',
      position: { x: 500, y: 150 },
    },
    {
      id: 'output-1',
      type: 'output',
      title: 'Final Decision',
      description: 'Decision output and execution',
      status: 'pending',
      position: { x: 700, y: 150 },
    },
  ];

  const edges: DecisionEdge[] = [
    { id: 'edge-1', sourceId: 'input-1', targetId: 'judge-1', type: 'judge_vote' },
    { id: 'edge-2', sourceId: 'input-1', targetId: 'judge-2', type: 'judge_vote' },
    { id: 'edge-3', sourceId: 'input-1', targetId: 'judge-3', type: 'judge_vote' },
    { id: 'edge-4', sourceId: 'judge-1', targetId: 'consensus-1', type: 'consensus_calc' },
    { id: 'edge-5', sourceId: 'judge-2', targetId: 'consensus-1', type: 'consensus_calc' },
    { id: 'edge-6', sourceId: 'judge-3', targetId: 'consensus-1', type: 'consensus_calc' },
    { id: 'edge-7', sourceId: 'consensus-1', targetId: 'output-1', type: 'final_decision' },
  ];

  const getNodeConfig = (type: string) => {
    switch (type) {
      case 'input':
        return { color: 'primary', icon: GitBranch };
      case 'judge':
        return { color: 'secondary', icon: Users };
      case 'consensus':
        return { color: 'warning', icon: AlertTriangle };
      case 'output':
        return { color: 'success', icon: CheckCircle };
      default:
        return { color: 'secondary', icon: Clock };
    }
  };

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'completed':
        return { color: 'success', icon: CheckCircle };
      case 'processing':
        return { color: 'warning', icon: Clock };
      case 'error':
        return { color: 'error', icon: XCircle };
      default:
        return { color: 'secondary', icon: Clock };
    }
  };

  const handleZoomIn = () => setZoom(Math.min(zoom * 1.2, 2));
  const handleZoomOut = () => setZoom(Math.max(zoom / 1.2, 0.5));
  const handleResetZoom = () => setZoom(1);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Decision Flow Visualization</Text>
          <Text variant="paragraph-small" color="secondary">
            Real-time view of council decision-making process
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.zoomControls}>
            <Button variant="secondary" size="sm" onClick={handleZoomOut}>
              <ZoomOut size={16} />
            </Button>
            <Text variant="paragraph-small" className={styles.zoomLevel}>
              {Math.round(zoom * 100)}%
            </Text>
            <Button variant="secondary" size="sm" onClick={handleZoomIn}>
              <ZoomIn size={16} />
            </Button>
            <Button variant="secondary" size="sm" onClick={handleResetZoom}>
              <RotateCcw size={16} />
            </Button>
          </div>
        </div>
      </div>

      {/* Active Decision Status */}
      <div className={styles.decisionStatus}>
        <div className={styles.decisionInfo}>
          <Text variant="h4" className={styles.decisionTitle}>
            {activeDecision.title}
          </Text>
          <div className={styles.decisionMeta}>
            <Badge variant="warning" size="sm">
              <Clock size={12} />
              <span>{activeDecision.status}</span>
            </Badge>
            <Text variant="paragraph-small" color="secondary">
              {activeDecision.currentStage}
            </Text>
          </div>
        </div>

        <div className={styles.decisionProgress}>
          <div className={styles.progressBar}>
            <div
              className={styles.progressFill}
              style={{ width: `${activeDecision.progress}%` }}
            />
          </div>
          <Text variant="paragraph-small" color="secondary">
            {activeDecision.progress}% complete
          </Text>
        </div>
      </div>

      {/* Flow Diagram */}
      <div className={styles.diagramContainer}>
        <div
          className={styles.diagram}
          style={{
            transform: `scale(${zoom})`,
            transformOrigin: 'top left'
          }}
        >
          {/* Render edges first (behind nodes) */}
          <svg className={styles.edges}>
            {edges.map((edge) => {
              const sourceNode = nodes.find(n => n.id === edge.sourceId);
              const targetNode = nodes.find(n => n.id === edge.targetId);

              if (!sourceNode || !targetNode) return null;

              const x1 = sourceNode.position.x + 120; // Node width / 2
              const y1 = sourceNode.position.y + 40;  // Node height / 2
              const x2 = targetNode.position.x + 120;
              const y2 = targetNode.position.y + 40;

              return (
                <g key={edge.id}>
                  <line
                    x1={x1}
                    y1={y1}
                    x2={x2}
                    y2={y2}
                    className={styles.edge}
                  />
                  <polygon
                    points={`${x2-6},${y2-3} ${x2-6},${y2+3} ${x2},${y2}`}
                    className={styles.edgeArrow}
                  />
                </g>
              );
            })}
          </svg>

          {/* Render nodes */}
          {nodes.map((node) => {
            const nodeConfig = getNodeConfig(node.type);
            const statusConfig = getStatusConfig(node.status);
            const NodeIcon = nodeConfig.icon;
            const StatusIcon = statusConfig.icon;

            return (
              <div
                key={node.id}
                className={`${styles.node} ${styles[node.type]}`}
                style={{
                  left: node.position.x,
                  top: node.position.y,
                }}
              >
                <div className={styles.nodeHeader}>
                  <div className={`${styles.nodeIcon} ${styles[nodeConfig.color]}`}>
                    <NodeIcon size={16} />
                  </div>
                  <div className={styles.nodeStatus}>
                    <StatusIcon size={12} className={styles[statusConfig.color]} />
                  </div>
                </div>

                <div className={styles.nodeContent}>
                  <Text variant="h5" className={styles.nodeTitle}>
                    {node.title}
                  </Text>
                  <Text variant="paragraph-small" color="secondary" className={styles.nodeDescription}>
                    {node.description}
                  </Text>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Judge Status Panel */}
      <div className={styles.judgePanel}>
        <Text variant="h4" className={styles.panelTitle}>Judge Status</Text>
        <div className={styles.judgeGrid}>
          {activeDecision.judges.map((judge) => {
            const statusConfig = getStatusConfig(judge.status);
            const StatusIcon = statusConfig.icon;

            return (
              <div key={judge.id} className={styles.judgeCard}>
                <div className={styles.judgeHeader}>
                  <Text variant="h5" className={styles.judgeName}>
                    {judge.name}
                  </Text>
                  <Badge variant={statusConfig.color as any} size="sm">
                    <StatusIcon size={12} />
                    <span>{judge.status}</span>
                  </Badge>
                </div>

                {judge.vote && (
                  <div className={styles.judgeVote}>
                    <Text variant="paragraph-small" color="secondary">
                      Vote: <strong>{judge.vote}</strong>
                    </Text>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Legend */}
      <div className={styles.legend}>
        <Text variant="h5" className={styles.legendTitle}>Legend</Text>
        <div className={styles.legendItems}>
          <div className={styles.legendItem}>
            <div className={`${styles.legendIcon} ${styles.primary}`}>
              <GitBranch size={14} />
            </div>
            <Text variant="paragraph-small">Task Input</Text>
          </div>

          <div className={styles.legendItem}>
            <div className={`${styles.legendIcon} ${styles.secondary}`}>
              <Users size={14} />
            </div>
            <Text variant="paragraph-small">Judge Evaluation</Text>
          </div>

          <div className={styles.legendItem}>
            <div className={`${styles.legendIcon} ${styles.warning}`}>
              <AlertTriangle size={14} />
            </div>
            <Text variant="paragraph-small">Consensus Calculation</Text>
          </div>

          <div className={styles.legendItem}>
            <div className={`${styles.legendIcon} ${styles.success}`}>
              <CheckCircle size={14} />
            </div>
            <Text variant="paragraph-small">Final Decision</Text>
          </div>
        </div>
      </div>
    </div>
  );
}
