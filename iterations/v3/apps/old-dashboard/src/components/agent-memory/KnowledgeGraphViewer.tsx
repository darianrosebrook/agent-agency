/**
 * Knowledge Graph Viewer
 * Interactive visualization of agent memory relationships and knowledge networks
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useRef } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  Network,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Filter,
  Download,
  RefreshCw,
  Brain,
  Database,
  User,
  FileText
} from 'lucide-react';
import { agentMemoryApiClient } from '@/lib/agent-memory-api';
import { useAgentMemoryStore, useAgentMemoryActions, useKnowledgeGraphStats } from '@/stores/agent-memory';
import { useAgentMemoryWebSocket } from '@/hooks/useAgentMemoryWebSocket';
import styles from './KnowledgeGraphViewer.module.scss';

// Simple D3-based graph visualization
interface GraphNode {
  id: string;
  x: number;
  y: number;
  fx?: number;
  fy?: number;
  memoryId: string;
  label: string;
  type: string;
  importance: number;
  confidence: number;
  size: number;
  color: string;
}

interface GraphLink {
  source: string;
  target: string;
  type: string;
  strength: number;
  color: string;
}

interface SimpleGraphProps {
  nodes: GraphNode[];
  links: GraphLink[];
  onNodeClick?: (node: GraphNode) => void;
  onLinkClick?: (link: GraphLink) => void;
}

const SimpleGraph: React.FC<SimpleGraphProps> = ({ nodes, links, onNodeClick, onLinkClick }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Simple force-directed layout simulation
  useEffect(() => {
    if (!nodes.length) return;

    const simulation = {
      nodes: [...nodes],
      links: [...links],
      forces: {
        link: (alpha: number) => {
          links.forEach(link => {
            const source = simulation.nodes.find(n => n.id === link.source);
            const target = simulation.nodes.find(n => n.id === link.target);
            if (!source || !target) return;

            const dx = target.x - source.x;
            const dy = target.y - source.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            const strength = link.strength * 50;
            const force = (distance - strength) * alpha * 0.1;

            if (distance > 0) {
              source.x += (dx / distance) * force;
              source.y += (dy / distance) * force;
              target.x -= (dx / distance) * force;
              target.y -= (dy / distance) * force;
            }
          });
        },
        charge: (alpha: number) => {
          simulation.nodes.forEach(node => {
            simulation.nodes.forEach(other => {
              if (node.id === other.id) return;

              const dx = other.x - node.x;
              const dy = other.y - node.y;
              const distance = Math.sqrt(dx * dx + dy * dy);
              const force = -1000 / (distance * distance) * alpha;

              if (distance > 0) {
                node.x += (dx / distance) * force;
                node.y += (dy / distance) * force;
              }
            });
          });
        },
        center: () => {
          const centerX = 400;
          const centerY = 300;
          simulation.nodes.forEach(node => {
            node.x += (centerX - node.x) * 0.01;
            node.y += (centerY - node.y) * 0.01;
          });
        }
      }
    };

    // Initialize positions
    simulation.nodes.forEach((node, i) => {
      if (node.fx === undefined) {
        node.x = Math.cos(i * 2 * Math.PI / simulation.nodes.length) * 200 + 400;
        node.y = Math.sin(i * 2 * Math.PI / simulation.nodes.length) * 200 + 300;
      }
    });

    // Run simulation
    let iterations = 0;
    const maxIterations = 100;
    const animate = () => {
      if (iterations >= maxIterations) return;

      simulation.forces.link(0.1);
      simulation.forces.charge(0.1);
      simulation.forces.center();

      // Update SVG
      if (svgRef.current) {
        const svg = svgRef.current;
        const nodeElements = svg.querySelectorAll('.node');
        const linkElements = svg.querySelectorAll('.link');

        simulation.nodes.forEach((node, i) => {
          const nodeEl = nodeElements[i] as SVGCircleElement;
          if (nodeEl) {
            nodeEl.setAttribute('cx', node.x.toString());
            nodeEl.setAttribute('cy', node.y.toString());
          }
        });

        simulation.links.forEach((link, i) => {
          const linkEl = linkElements[i] as SVGLineElement;
          if (linkEl) {
            const source = simulation.nodes.find(n => n.id === link.source);
            const target = simulation.nodes.find(n => n.id === link.target);
            if (source && target) {
              linkEl.setAttribute('x1', source.x.toString());
              linkEl.setAttribute('y1', source.y.toString());
              linkEl.setAttribute('x2', target.x.toString());
              linkEl.setAttribute('y2', target.y.toString());
            }
          }
        });
      }

      iterations++;
      requestAnimationFrame(animate);
    };

    animate();
  }, [nodes, links]);

  const handleZoomIn = () => setZoom(Math.min(zoom * 1.2, 3));
  const handleZoomOut = () => setZoom(Math.max(zoom / 1.2, 0.3));
  const handleReset = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    setDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (dragging) {
      setPan({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y
      });
    }
  };

  const handleMouseUp = () => {
    setDragging(false);
  };

  return (
    <div className={styles.graphContainer}>
      <div className={styles.graphControls}>
        <Button variant="secondary" size="sm" onClick={handleZoomIn}>
          <ZoomIn size={14} />
        </Button>
        <Button variant="secondary" size="sm" onClick={handleZoomOut}>
          <ZoomOut size={14} />
        </Button>
        <Button variant="secondary" size="sm" onClick={handleReset}>
          <RotateCcw size={14} />
        </Button>
        <span className={styles.zoomLevel}>{Math.round(zoom * 100)}%</span>
      </div>

      <svg
        ref={svgRef}
        className={styles.graphSvg}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        style={{
          transform: `scale(${zoom}) translate(${pan.x / zoom}px, ${pan.y / zoom}px)`,
          cursor: dragging ? 'grabbing' : 'grab'
        }}
      >
        {/* Links */}
        {links.map((link, i) => (
          <line
            key={`link-${i}`}
            className="link"
            stroke={link.color}
            strokeWidth={Math.max(link.strength * 3, 1)}
            opacity={0.6}
            onClick={() => onLinkClick?.(link)}
          />
        ))}

        {/* Nodes */}
        {nodes.map((node, i) => (
          <g key={`node-${i}`}>
            <circle
              className="node"
              r={node.size}
              fill={node.color}
              stroke="#fff"
              strokeWidth={2}
              onClick={() => onNodeClick?.(node)}
              style={{ cursor: 'pointer' }}
            />
            <text
              x={node.x}
              y={node.y - node.size - 5}
              textAnchor="middle"
              fontSize="12"
              fill="#333"
              pointerEvents="none"
            >
              {node.label.length > 15 ? `${node.label.substring(0, 15)}...` : node.label}
            </text>
          </g>
        ))}
      </svg>

      {nodes.length === 0 && (
        <div className={styles.emptyGraph}>
          <Network size={48} />
          <Text variant="h3">No Knowledge Graph Data</Text>
          <Text variant="paragraph-medium" color="secondary">
            Generate a knowledge graph to visualize memory relationships.
          </Text>
        </div>
      )}
    </div>
  );
};

export function KnowledgeGraphViewer() {
  const [selectedAgent, setSelectedAgent] = useState<string>('all');
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [selectedLink, setSelectedLink] = useState<GraphLink | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [graphLayout, setGraphLayout] = useState<'force' | 'hierarchical' | 'circular'>('force');
  const [minImportance, setMinImportance] = useState(0.1);
  const [maxNodes, setMaxNodes] = useState(100);

  const { agents } = useAgentMemoryStore();
  const actions = useAgentMemoryActions();
  const {} = useAgentMemoryWebSocket();

  const graphStats = useKnowledgeGraphStats();

  // Mock graph data for demonstration
  const mockGraphData = {
    nodes: [
      {
        id: 'node-1',
        x: 400,
        y: 300,
        memoryId: 'mem-1',
        label: 'Climate Analysis',
        type: 'conversation',
        importance: 0.85,
        confidence: 0.92,
        size: 15,
        color: '#3B82F6'
      },
      {
        id: 'node-2',
        x: 500,
        y: 200,
        memoryId: 'mem-2',
        label: 'ML Validation',
        type: 'knowledge',
        importance: 0.78,
        confidence: 0.95,
        size: 12,
        color: '#F59E0B'
      },
      {
        id: 'node-3',
        x: 300,
        y: 400,
        memoryId: 'mem-3',
        label: 'Pipeline Execution',
        type: 'experience',
        importance: 0.92,
        confidence: 0.88,
        size: 18,
        color: '#EF4444'
      },
      {
        id: 'node-4',
        x: 600,
        y: 350,
        memoryId: 'mem-4',
        label: 'Data Facts',
        type: 'fact',
        importance: 0.65,
        confidence: 0.89,
        size: 10,
        color: '#10B981'
      }
    ],
    links: [
      {
        source: 'node-1',
        target: 'node-2',
        type: 'related',
        strength: 0.7,
        color: '#94A3B8'
      },
      {
        source: 'node-2',
        target: 'node-3',
        type: 'supports',
        strength: 0.8,
        color: '#94A3B8'
      },
      {
        source: 'node-3',
        target: 'node-4',
        type: 'uses',
        strength: 0.6,
        color: '#94A3B8'
      },
      {
        source: 'node-1',
        target: 'node-4',
        type: 'similar',
        strength: 0.5,
        color: '#94A3B8'
      }
    ]
  };

  const currentGraph = mockGraphData; // Replace with real data from store

  // Fetch graph data
  useEffect(() => {
    const fetchGraphData = async () => {
      try {
        actions.setLoading('knowledgeGraph', true);
        // Mock API call - replace with real implementation
        console.log('Fetching knowledge graph for agent:', selectedAgent);
        // const graphData = await agentMemoryApiClient.getKnowledgeGraph(selectedAgent === 'all' ? undefined : selectedAgent);
        // actions.updateKnowledgeGraph(selectedAgent, graphData);
      } catch (error) {
        console.error('Failed to fetch knowledge graph:', error);
        actions.setError('knowledgeGraph', error instanceof Error ? error.message : 'Failed to fetch knowledge graph');
      } finally {
        actions.setLoading('knowledgeGraph', false);
      }
    };

    fetchGraphData();
  }, [selectedAgent]);

  const handleNodeClick = (node: GraphNode) => {
    setSelectedNode(node);
    setSelectedLink(null);
  };

  const handleLinkClick = (link: GraphLink) => {
    setSelectedLink(link);
    setSelectedNode(null);
  };

  const handleGenerateGraph = async () => {
    try {
      actions.setLoading('knowledgeGraph', true);
      const graphData = await agentMemoryApiClient.getKnowledgeGraph(
        selectedAgent === 'all' ? undefined : selectedAgent,
        {
          maxNodes,
          minImportance,
          layout: graphLayout
        }
      );
      actions.updateKnowledgeGraph(selectedAgent, graphData);
    } catch (error) {
      console.error('Failed to generate knowledge graph:', error);
    } finally {
      actions.setLoading('knowledgeGraph', false);
    }
  };

  const handleExportGraph = () => {
    // Export graph as JSON or image
    const dataStr = JSON.stringify(currentGraph, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);

    const exportFileDefaultName = `knowledge-graph-${selectedAgent}-${new Date().toISOString().split('T')[0]}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  const agentOptions = [
    { value: 'all', label: 'All Agents', count: Object.keys(graphStats).length },
    ...agents.map((agent, index) => ({
      value: agent.agentId || `agent-${index}`,
      label: agent.name,
      count: graphStats[agent.agentId || `agent-${index}`]?.nodes || 0
    }))
  ];

  return (
    <div className={styles.knowledgeGraphViewer}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Knowledge Graph Viewer</Text>
          <Text variant="paragraph-large" color="secondary">
            Interactive visualization of agent memory relationships and knowledge networks
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Graph Stats */}
          <div className={styles.graphStats}>
            <div className={styles.stat}>
              <Text variant="h3">{currentGraph.nodes.length}</Text>
              <Text variant="paragraph-small" color="secondary">Nodes</Text>
            </div>
            <div className={styles.stat}>
              <Text variant="h3">{currentGraph.links.length}</Text>
              <Text variant="paragraph-small" color="secondary">Connections</Text>
            </div>
            <div className={styles.stat}>
              <Text variant="h3">
                {currentGraph.nodes.length > 0
                  ? (currentGraph.links.length / currentGraph.nodes.length).toFixed(1)
                  : '0'
                }
              </Text>
              <Text variant="paragraph-small" color="secondary">Avg Degree</Text>
            </div>
          </div>
        </div>
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.controlGroup}>
          <label>Agent:</label>
          <select
            value={selectedAgent}
            onChange={(e) => setSelectedAgent(e.target.value)}
            className={styles.select}
          >
            {agentOptions.map(option => (
              <option key={option.value} value={option.value}>
                {option.label} ({option.count})
              </option>
            ))}
          </select>
        </div>

        <div className={styles.controlGroup}>
          <label>Layout:</label>
          <select
            value={graphLayout}
            onChange={(e) => setGraphLayout(e.target.value as any)}
            className={styles.select}
          >
            <option value="force">Force Directed</option>
            <option value="hierarchical">Hierarchical</option>
            <option value="circular">Circular</option>
          </select>
        </div>

        <div className={styles.controlGroup}>
          <label>Min Importance:</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={minImportance}
            onChange={(e) => setMinImportance(parseFloat(e.target.value))}
            className={styles.slider}
          />
          <span className={styles.sliderValue}>{minImportance.toFixed(1)}</span>
        </div>

        <div className={styles.controlGroup}>
          <label>Max Nodes:</label>
          <input
            type="number"
            min="10"
            max="1000"
            value={maxNodes}
            onChange={(e) => setMaxNodes(parseInt(e.target.value))}
            className={styles.numberInput}
          />
        </div>

        <div className={styles.actionButtons}>
          <Button variant="secondary" size="sm" onClick={() => setShowFilters(!showFilters)}>
            <Filter size={14} />
            Filters
          </Button>
          <Button variant="primary" size="sm" onClick={handleGenerateGraph}>
            <RefreshCw size={14} />
            Generate
          </Button>
          <Button variant="secondary" size="sm" onClick={handleExportGraph}>
            <Download size={14} />
            Export
          </Button>
        </div>
      </div>

      {/* Graph Container */}
      <div className={styles.graphWrapper}>
        <SimpleGraph
          nodes={currentGraph.nodes}
          links={currentGraph.links}
          onNodeClick={handleNodeClick}
          onLinkClick={handleLinkClick}
        />

        {/* Legend */}
        <div className={styles.legend}>
          <Text variant="h4">Node Types</Text>
          <div className={styles.legendItems}>
            <div className={styles.legendItem}>
              <div className={styles.legendColor} style={{ backgroundColor: '#3B82F6' }}></div>
              <Text variant="paragraph-small">Conversation</Text>
            </div>
            <div className={styles.legendItem}>
              <div className={styles.legendColor} style={{ backgroundColor: '#10B981' }}></div>
              <Text variant="paragraph-small">Facts</Text>
            </div>
            <div className={styles.legendItem}>
              <div className={styles.legendColor} style={{ backgroundColor: '#F59E0B' }}></div>
              <Text variant="paragraph-small">Knowledge</Text>
            </div>
            <div className={styles.legendItem}>
              <div className={styles.legendColor} style={{ backgroundColor: '#EF4444' }}></div>
              <Text variant="paragraph-small">Experience</Text>
            </div>
          </div>
        </div>
      </div>

      {/* Selection Details */}
      {(selectedNode || selectedLink) && (
        <div className={styles.selectionDetails}>
          {selectedNode && (
            <div className={styles.detailCard}>
              <div className={styles.detailHeader}>
                <div className={styles.nodeIcon}>
                  {selectedNode.type === 'conversation' && <User size={16} />}
                  {selectedNode.type === 'fact' && <FileText size={16} />}
                  {selectedNode.type === 'knowledge' && <Brain size={16} />}
                  {selectedNode.type === 'experience' && <Database size={16} />}
                </div>
                <Text variant="h4">{selectedNode.label}</Text>
              </div>

              <div className={styles.detailGrid}>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Type</Text>
                  <Text variant="paragraph-medium">{selectedNode.type}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Importance</Text>
                  <Text variant="paragraph-medium">{selectedNode.importance.toFixed(2)}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Confidence</Text>
                  <Text variant="paragraph-medium">{selectedNode.confidence.toFixed(2)}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Memory ID</Text>
                  <Text variant="paragraph-medium">{selectedNode.memoryId}</Text>
                </div>
              </div>
            </div>
          )}

          {selectedLink && (
            <div className={styles.detailCard}>
              <div className={styles.detailHeader}>
                <Network size={16} />
                <Text variant="h4">Connection</Text>
              </div>

              <div className={styles.detailGrid}>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Type</Text>
                  <Text variant="paragraph-medium">{selectedLink.type}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">Strength</Text>
                  <Text variant="paragraph-medium">{selectedLink.strength.toFixed(2)}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">From</Text>
                  <Text variant="paragraph-medium">{selectedLink.source}</Text>
                </div>
                <div className={styles.detailItem}>
                  <Text variant="paragraph-small" color="secondary">To</Text>
                  <Text variant="paragraph-medium">{selectedLink.target}</Text>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
