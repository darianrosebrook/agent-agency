/**
 * D3Visualization Component
 * Wrapper for D3.js visualizations with React integration
 *
 * @author @darianrosebrook
 */

'use client';

import { useEffect, useRef, useState } from 'react';
import { Text } from '@/design-system/primitives';
import * as d3 from 'd3';
import styles from './D3Visualization.module.scss';

export interface D3VisualizationProps {
  title?: string;
  subtitle?: string;
  data: any[];
  width?: number;
  height?: number;
  type: 'line' | 'bar' | 'scatter' | 'heatmap' | 'network' | 'custom';
  config?: {
    xAxis?: string;
    yAxis?: string;
    color?: string;
    size?: string;
    [key: string]: any;
  };
  onDataPointClick?: (data: any) => void;
  onDataPointHover?: (data: any) => void;
  className?: string;
  children?: (d3Ref: React.RefObject<HTMLDivElement>) => React.ReactNode;
}

export function D3Visualization({
  title,
  subtitle,
  data,
  width = 400,
  height = 300,
  type,
  config = {},
  onDataPointClick,
  onDataPointHover,
  className = '',
  children,
}: D3VisualizationProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width, height });
  const [isLoading, setIsLoading] = useState(true);

  // Responsive dimensions
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.offsetWidth;
        const newWidth = Math.min(width, containerWidth - 32); // Account for padding
        setDimensions({ width: newWidth, height });
      }
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, [width, height]);

  // D3 visualization rendering
  useEffect(() => {
    if (!svgRef.current || !data.length) {
      setIsLoading(false);
      return;
    }

    setIsLoading(true);

    // Clear previous content
    d3.select(svgRef.current).selectAll('*').remove();

    const svg = d3.select(svgRef.current);
    const { width: svgWidth, height: svgHeight } = dimensions;

    // Set up scales and axes
    const margin = { top: 20, right: 20, bottom: 40, left: 40 };
    const innerWidth = svgWidth - margin.left - margin.right;
    const innerHeight = svgHeight - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create scales based on data
    const xScale = d3.scaleLinear()
      .domain(d3.extent(data, d => d[config.xAxis || 'x']) as [number, number])
      .range([0, innerWidth]);

    const yScale = d3.scaleLinear()
      .domain(d3.extent(data, d => d[config.yAxis || 'y']) as [number, number])
      .range([innerHeight, 0]);

    const colorScale = d3.scaleOrdinal()
      .domain([...new Set(data.map(d => d[config.color || 'category']))])
      .range(d3.schemeCategory10);

    // Render based on type
    switch (type) {
      case 'line':
        renderLineChart(g, data, xScale, yScale, colorScale, config);
        break;
      case 'bar':
        renderBarChart(g, data, xScale, yScale, colorScale, config);
        break;
      case 'scatter':
        renderScatterPlot(g, data, xScale, yScale, colorScale, config);
        break;
      case 'heatmap':
        renderHeatmap(g, data, xScale, yScale, colorScale, config);
        break;
      case 'network':
        renderNetwork(g, data, xScale, yScale, colorScale, config);
        break;
      default:
        // Custom rendering - let children handle it
        break;
    }

    // Add axes
    g.append('g')
      .attr('transform', `translate(0,${innerHeight})`)
      .call(d3.axisBottom(xScale));

    g.append('g')
      .call(d3.axisLeft(yScale));

    setIsLoading(false);
  }, [data, dimensions, type, config]);

  // Event handlers
  const handleDataPointClick = (event: React.MouseEvent, data: any) => {
    onDataPointClick?.(data);
  };

  const handleDataPointHover = (event: React.MouseEvent, data: any) => {
    onDataPointHover?.(data);
  };

  return (
    <div className={`${styles.d3Visualization} ${className}`}>
      {(title || subtitle) && (
        <div className={styles.header}>
          {title && (
            <Text variant="h4" className={styles.title}>
              {title}
            </Text>
          )}
          {subtitle && (
            <Text variant="paragraph-small" color="secondary" className={styles.subtitle}>
              {subtitle}
            </Text>
          )}
        </div>
      )}

      <div className={styles.container} ref={containerRef}>
        {isLoading && (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <Text variant="paragraph-small" color="secondary">
              Loading visualization...
            </Text>
          </div>
        )}

        <svg
          ref={svgRef}
          width={dimensions.width}
          height={dimensions.height}
          className={styles.svg}
        />

        {children && children(containerRef)}
      </div>
    </div>
  );
}

// D3 rendering functions
function renderLineChart(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  data: any[],
  xScale: d3.ScaleLinear<number, number>,
  yScale: d3.ScaleLinear<number, number>,
  colorScale: d3.ScaleOrdinal<string, string, never>,
  config: any
) {
  const line = d3.line<any>()
    .x(d => xScale(d[config.xAxis || 'x']))
    .y(d => yScale(d[config.yAxis || 'y']))
    .curve(d3.curveMonotoneX);

  g.append('path')
    .datum(data)
    .attr('fill', 'none')
    .attr('stroke', colorScale('default'))
    .attr('stroke-width', 2)
    .attr('d', line);
}

function renderBarChart(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  data: any[],
  xScale: d3.ScaleLinear<number, number>,
  yScale: d3.ScaleLinear<number, number>,
  colorScale: d3.ScaleOrdinal<string, string, never>,
  config: any
) {
  g.selectAll('.bar')
    .data(data)
    .enter()
    .append('rect')
    .attr('class', 'bar')
    .attr('x', d => xScale(d[config.xAxis || 'x']))
    .attr('y', d => yScale(d[config.yAxis || 'y']))
    .attr('width', xScale.bandwidth ? xScale.bandwidth() : 20)
    .attr('height', d => yScale(0) - yScale(d[config.yAxis || 'y']))
    .attr('fill', d => colorScale(d[config.color || 'category']));
}

function renderScatterPlot(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  data: any[],
  xScale: d3.ScaleLinear<number, number>,
  yScale: d3.ScaleLinear<number, number>,
  colorScale: d3.ScaleOrdinal<string, string, never>,
  config: any
) {
  g.selectAll('.dot')
    .data(data)
    .enter()
    .append('circle')
    .attr('class', 'dot')
    .attr('cx', d => xScale(d[config.xAxis || 'x']))
    .attr('cy', d => yScale(d[config.yAxis || 'y']))
    .attr('r', d => (d[config.size || 'size'] || 4))
    .attr('fill', d => colorScale(d[config.color || 'category']))
    .attr('opacity', 0.7);
}

function renderHeatmap(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  data: any[],
  xScale: d3.ScaleLinear<number, number>,
  yScale: d3.ScaleLinear<number, number>,
  colorScale: d3.ScaleOrdinal<string, string, never>,
  config: any
) {
  const colorIntensity = d3.scaleLinear()
    .domain(d3.extent(data, d => d[config.intensity || 'value']) as [number, number])
    .range([0, 1]);

  g.selectAll('.cell')
    .data(data)
    .enter()
    .append('rect')
    .attr('class', 'cell')
    .attr('x', d => xScale(d[config.xAxis || 'x']))
    .attr('y', d => yScale(d[config.yAxis || 'y']))
    .attr('width', xScale.bandwidth ? xScale.bandwidth() : 20)
    .attr('height', yScale.bandwidth ? yScale.bandwidth() : 20)
    .attr('fill', d => d3.interpolateViridis(colorIntensity(d[config.intensity || 'value'])));
}

function renderNetwork(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  data: any[],
  xScale: d3.ScaleLinear<number, number>,
  yScale: d3.ScaleLinear<number, number>,
  colorScale: d3.ScaleOrdinal<string, string, never>,
  config: any
) {
  // Simple network visualization
  const nodes = data.map(d => ({ id: d.id, x: d.x, y: d.y, group: d.group }));
  const links = data.flatMap(d => d.connections || []);

  // Render links
  g.selectAll('.link')
    .data(links)
    .enter()
    .append('line')
    .attr('class', 'link')
    .attr('x1', d => xScale(d.source.x))
    .attr('y1', d => yScale(d.source.y))
    .attr('x2', d => xScale(d.target.x))
    .attr('y2', d => yScale(d.target.y))
    .attr('stroke', '#999')
    .attr('stroke-opacity', 0.6)
    .attr('stroke-width', 1);

  // Render nodes
  g.selectAll('.node')
    .data(nodes)
    .enter()
    .append('circle')
    .attr('class', 'node')
    .attr('cx', d => xScale(d.x))
    .attr('cy', d => yScale(d.y))
    .attr('r', 5)
    .attr('fill', d => colorScale(d.group))
    .attr('stroke', '#fff')
    .attr('stroke-width', 2);
}
