/**
 * Vector3DVisualization Component
 * 3D visualization for high-dimensional vector data using Three.js
 *
 * @author @darianrosebrook
 */

'use client';

import { useRef, useEffect, useState } from 'react';
import { Canvas, useFrame, useThree } from '@react-three/fiber';
import { OrbitControls, Text, Sphere, Line } from '@react-three/drei';
import { Text as TextPrimitive } from '@/design-system/primitives';
import * as THREE from 'three';
import styles from './Vector3DVisualization.module.scss';

export interface Vector3DVisualizationProps {
  title?: string;
  subtitle?: string;
  vectors: {
    id: string;
    position: [number, number, number];
    embedding: number[];
    label?: string;
    category?: string;
    metadata?: Record<string, any>;
  }[];
  clusters?: {
    id: string;
    center: [number, number, number];
    radius: number;
    label: string;
    color: string;
  }[];
  projection?: 'pca' | 'tsne' | 'umap' | 'custom';
  onVectorClick?: (vector: any) => void;
  onVectorHover?: (vector: any) => void;
  className?: string;
}

interface VectorPointProps {
  position: [number, number, number];
  vector: any;
  onClick?: (vector: any) => void;
  onHover?: (vector: any) => void;
}

function VectorPoint({ position, vector, onClick, onHover }: VectorPointProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const [hovered, setHovered] = useState(false);

  useFrame((state) => {
    if (meshRef.current) {
      meshRef.current.scale.setScalar(hovered ? 1.2 : 1);
      meshRef.current.rotation.y += 0.01;
    }
  });

  return (
    <Sphere
      ref={meshRef}
      position={position}
      args={[0.1, 16, 16]}
      onClick={() => onClick?.(vector)}
      onPointerOver={() => {
        setHovered(true);
        onHover?.(vector);
      }}
      onPointerOut={() => setHovered(false)}
    >
      <meshStandardMaterial
        color={vector.category ? getCategoryColor(vector.category) : '#6366f1'}
        emissive={hovered ? '#818cf8' : '#000'}
        emissiveIntensity={hovered ? 0.3 : 0}
        roughness={0.3}
        metalness={0.7}
      />
    </Sphere>
  );
}

interface ClusterSphereProps {
  center: [number, number, number];
  radius: number;
  color: string;
  label: string;
}

function ClusterSphere({ center, radius, color, label }: ClusterSphereProps) {
  const meshRef = useRef<THREE.Mesh>(null);

  useFrame((state) => {
    if (meshRef.current) {
      meshRef.current.rotation.y += 0.005;
    }
  });

  return (
    <group>
      <Sphere
        ref={meshRef}
        position={center}
        args={[radius, 32, 32]}
      >
        <meshStandardMaterial
          color={color}
          transparent
          opacity={0.1}
          wireframe
        />
      </Sphere>
      <Text
        position={[center[0], center[1] + radius + 0.2, center[2]]}
        fontSize={0.3}
        color={color}
        anchorX="center"
        anchorY="middle"
      >
        {label}
      </Text>
    </group>
  );
}

interface VectorConnectionsProps {
  vectors: any[];
  maxConnections?: number;
}

function VectorConnections({ vectors, maxConnections = 10 }: VectorConnectionsProps) {
  const connections = useRef<THREE.Line[]>([]);

  useEffect(() => {
    // Calculate connections based on similarity
    const similarityThreshold = 0.8;
    const newConnections: THREE.Line[] = [];

    for (let i = 0; i < vectors.length && newConnections.length < maxConnections; i++) {
      for (let j = i + 1; j < vectors.length && newConnections.length < maxConnections; j++) {
        const similarity = calculateCosineSimilarity(vectors[i].embedding, vectors[j].embedding);
        if (similarity > similarityThreshold) {
          const line = new THREE.Line(
            new THREE.BufferGeometry().setFromPoints([
              new THREE.Vector3(...vectors[i].position),
              new THREE.Vector3(...vectors[j].position)
            ]),
            new THREE.LineBasicMaterial({
              color: '#6366f1',
              opacity: similarity * 0.5,
              transparent: true
            })
          );
          newConnections.push(line);
        }
      }
    }

    connections.current = newConnections;
  }, [vectors, maxConnections]);

  return (
    <group>
      {connections.current.map((line, index) => (
        <primitive key={index} object={line} />
      ))}
    </group>
  );
}

function Scene({ vectors, clusters, onVectorClick, onVectorHover }: {
  vectors: any[];
  clusters?: any[];
  onVectorClick?: (vector: any) => void;
  onVectorHover?: (vector: any) => void;
}) {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(5, 5, 5);
    camera.lookAt(0, 0, 0);
  }, [camera]);

  return (
    <>
      <ambientLight intensity={0.4} />
      <directionalLight position={[10, 10, 5]} intensity={1} />
      <pointLight position={[-10, -10, -5]} intensity={0.5} />

      {/* Grid */}
      <gridHelper args={[10, 10, '#374151', '#374151']} />

      {/* Vector points */}
      {vectors.map((vector) => (
        <VectorPoint
          key={vector.id}
          position={vector.position}
          vector={vector}
          onClick={onVectorClick}
          onHover={onVectorHover}
        />
      ))}

      {/* Cluster spheres */}
      {clusters?.map((cluster) => (
        <ClusterSphere
          key={cluster.id}
          center={cluster.center}
          radius={cluster.radius}
          color={cluster.color}
          label={cluster.label}
        />
      ))}

      {/* Vector connections */}
      <VectorConnections vectors={vectors} />

      <OrbitControls
        enablePan={true}
        enableZoom={true}
        enableRotate={true}
        dampingFactor={0.05}
        screenSpacePanning={false}
        minDistance={2}
        maxDistance={20}
      />
    </>
  );
}

export function Vector3DVisualization({
  title,
  subtitle,
  vectors,
  clusters,
  projection = 'pca',
  onVectorClick,
  onVectorHover,
  className = '',
}: Vector3DVisualizationProps) {
  const [selectedVector, setSelectedVector] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [processedVectors, setProcessedVectors] = useState<any[]>([]);

  // Process vectors for 3D projection
  useEffect(() => {
    if (vectors.length === 0) {
      setIsLoading(false);
      setProcessedVectors([]);
      return;
    }

    setIsLoading(true);

    // Apply dimensionality reduction if needed
    const processed = vectors.map(vector => {
      // Use existing position if available, otherwise project from embedding
      const position = vector.position || projectTo3D(vector.embedding, projection);
      return {
        ...vector,
        position
      };
    });

    setProcessedVectors(processed);
    setIsLoading(false);
  }, [vectors, projection]);

  const handleVectorClick = (vector: any) => {
    setSelectedVector(vector);
    onVectorClick?.(vector);
  };

  const handleVectorHover = (vector: any) => {
    onVectorHover?.(vector);
  };

  return (
    <div className={`${styles.vector3DVisualization} ${className}`}>
      {(title || subtitle) && (
        <div className={styles.header}>
          {title && (
            <TextPrimitive variant="h4" className={styles.title}>
              {title}
            </TextPrimitive>
          )}
          {subtitle && (
            <TextPrimitive variant="paragraph-small" color="secondary" className={styles.subtitle}>
              {subtitle}
            </TextPrimitive>
          )}
        </div>
      )}

      <div className={styles.container}>
        {isLoading && (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <TextPrimitive variant="paragraph-small" color="secondary">
              Processing {vectors.length} vectors...
            </TextPrimitive>
          </div>
        )}

        <div className={styles.canvasContainer}>
          {processedVectors.length > 0 && (
            <Canvas
              camera={{ position: [5, 5, 5], fov: 60 }}
              style={{ width: '100%', height: '100%' }}
              gl={{ antialias: true, alpha: true, preserveDrawingBuffer: true }}
            >
              <Scene
                vectors={processedVectors}
                clusters={clusters}
                onVectorClick={handleVectorClick}
                onVectorHover={handleVectorHover}
              />
            </Canvas>
          )}
        </div>

        {/* Vector details panel */}
        {selectedVector && (
          <div className={styles.detailsPanel}>
            <div className={styles.detailsHeader}>
              <TextPrimitive variant="h5">Vector Details</TextPrimitive>
              <button
                className={styles.closeButton}
                onClick={() => setSelectedVector(null)}
              >
                ×
              </button>
            </div>
            
            <div className={styles.detailsContent}>
              <div className={styles.detailItem}>
                <TextPrimitive variant="paragraph-small" color="secondary">ID</TextPrimitive>
                <TextPrimitive variant="paragraph-medium">{selectedVector.id}</TextPrimitive>
              </div>
              
              {selectedVector.label && (
                <div className={styles.detailItem}>
                  <TextPrimitive variant="paragraph-small" color="secondary">Label</TextPrimitive>
                  <TextPrimitive variant="paragraph-medium">{selectedVector.label}</TextPrimitive>
                </div>
              )}
              
              {selectedVector.category && (
                <div className={styles.detailItem}>
                  <TextPrimitive variant="paragraph-small" color="secondary">Category</TextPrimitive>
                  <TextPrimitive variant="paragraph-medium">{selectedVector.category}</TextPrimitive>
                </div>
              )}
              
              <div className={styles.detailItem}>
                <TextPrimitive variant="paragraph-small" color="secondary">Position</TextPrimitive>
                <TextPrimitive variant="paragraph-medium">
                  [{selectedVector.position.map((p: number) => p.toFixed(2)).join(', ')}]
                </TextPrimitive>
              </div>
              
              <div className={styles.detailItem}>
                <TextPrimitive variant="paragraph-small" color="secondary">Dimensions</TextPrimitive>
                <TextPrimitive variant="paragraph-medium">{selectedVector.embedding.length}</TextPrimitive>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Helper functions
function getCategoryColor(category: string): string {
  const colors = {
    'neural': '#8b5cf6',
    'transformer': '#06b6d4',
    'cnn': '#10b981',
    'rnn': '#f59e0b',
    'attention': '#ef4444',
    'embedding': '#6366f1',
    'default': '#6b7280'
  };
  return colors[category as keyof typeof colors] || colors.default;
}

function calculateCosineSimilarity(a: number[], b: number[]): number {
  if (a.length !== b.length) return 0;
  
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  
  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }
  
  return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
}

function projectTo3D(embedding: number[], method: string): [number, number, number] {
  // Simple PCA-like projection for demonstration
  // In production, you'd use proper dimensionality reduction libraries
  
  if (embedding.length < 3) {
    return [embedding[0] || 0, embedding[1] || 0, 0];
  }
  
  // Use first 3 dimensions as x, y, z
  return [
    embedding[0] * 2,
    embedding[1] * 2,
    embedding[2] * 2
  ];
}
