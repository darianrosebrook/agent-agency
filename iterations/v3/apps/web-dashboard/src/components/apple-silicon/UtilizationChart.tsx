/**
 * Utilization Chart
 * Real-time hardware utilization visualization
 *
 * @author @darianrosebrook
 */

'use client';

import { useEffect, useRef } from 'react';
import { Text } from '@/design-system/primitives';
import * as d3 from 'd3';
import styles from './UtilizationChart.module.scss';

interface ChartData {
  timestamp: Date;
  ane: number;
  gpu: number;
  cpu: number;
  memory: number;
  power: number;
  temperature: number;
}

interface UtilizationChartProps {
  data: ChartData[];
  type: 'line' | 'area' | 'bar';
  timeRange: '1h' | '6h' | '24h';
}

export function UtilizationChart({ data, type, timeRange }: UtilizationChartProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data.length) return;

    // Clear previous chart
    d3.select(svgRef.current).selectAll('*').remove();

    // Chart dimensions
    const margin = { top: 20, right: 80, bottom: 40, left: 60 };
    const width = 800 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    // Create SVG
    const svg = d3.select(svgRef.current)
      .attr('width', width + margin.left + margin.right)
      .attr('height', height + margin.top + margin.bottom)
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Scales
    const xScale = d3.scaleTime()
      .domain(d3.extent(data, d => d.timestamp) as [Date, Date])
      .range([0, width]);

    const yScale = d3.scaleLinear()
      .domain([0, 100])
      .range([height, 0]);

    // Color scale for different metrics
    const colorScale = d3.scaleOrdinal<string>()
      .domain(['ane', 'gpu', 'cpu', 'memory'])
      .range(['#3B82F6', '#10B981', '#F59E0B', '#EF4444']);

    // Axes
    const xAxis = d3.axisBottom(xScale)
      .ticks(5)
      .tickFormat(d => {
        const date = d as Date;
        return timeRange === '1h' ? d3.timeFormat('%H:%M')(date) :
               timeRange === '6h' ? d3.timeFormat('%H:%M')(date) :
               d3.timeFormat('%m/%d %H:%M')(date);
      });

    const yAxis = d3.axisLeft(yScale)
      .ticks(5)
      .tickFormat(d => `${d}%`);

    svg.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${height})`)
      .call(xAxis);

    svg.append('g')
      .attr('class', 'y-axis')
      .call(yAxis);

    // Grid lines
    svg.append('g')
      .attr('class', 'grid')
      .attr('opacity', 0.1)
      .call(d3.axisLeft(yScale)
        .tickSize(-width)
        .tickFormat(() => ''));

    // Metrics to display
    const metrics = [
      { key: 'ane', label: 'ANE', color: '#3B82F6' },
      { key: 'gpu', label: 'GPU', color: '#10B981' },
      { key: 'cpu', label: 'CPU', color: '#F59E0B' },
      { key: 'memory', label: 'Memory', color: '#EF4444' }
    ];

    // Tooltip
    const tooltip = d3.select(tooltipRef.current);

    // Draw chart based on type
    if (type === 'line' || type === 'area') {
      const line = d3.line<ChartData>()
        .x(d => xScale(d.timestamp))
        .y(d => yScale(d[type === 'line' ? metrics.find(m => m.key === 'ane')?.key as keyof ChartData : 'ane' as keyof ChartData] as number))
        .curve(d3.curveMonotoneX);

      metrics.forEach(metric => {
        const metricLine = d3.line<ChartData>()
          .x(d => xScale(d.timestamp))
          .y(d => yScale(d[metric.key as keyof ChartData] as number))
          .curve(d3.curveMonotoneX);

        if (type === 'area') {
          const area = d3.area<ChartData>()
            .x(d => xScale(d.timestamp))
            .y0(height)
            .y1(d => yScale(d[metric.key as keyof ChartData] as number))
            .curve(d3.curveMonotoneX);

          svg.append('path')
            .datum(data)
            .attr('fill', metric.color)
            .attr('fill-opacity', 0.1)
            .attr('d', area);
        }

        svg.append('path')
          .datum(data)
          .attr('fill', 'none')
          .attr('stroke', metric.color)
          .attr('stroke-width', 2)
          .attr('d', metricLine)
          .on('mouseover', function() {
            d3.select(this).attr('stroke-width', 3);
          })
          .on('mouseout', function() {
            d3.select(this).attr('stroke-width', 2);
          });
      });

      // Add interactive circles for hover
      metrics.forEach(metric => {
        svg.selectAll(`.dot-${metric.key}`)
          .data(data)
          .enter()
          .append('circle')
          .attr('class', `dot-${metric.key}`)
          .attr('cx', d => xScale(d.timestamp))
          .attr('cy', d => yScale(d[metric.key as keyof ChartData] as number))
          .attr('r', 3)
          .attr('fill', metric.color)
          .attr('opacity', 0)
          .on('mouseover', function(event, d) {
            d3.select(this).attr('opacity', 1);

            tooltip
              .style('opacity', 1)
              .style('left', `${event.pageX + 10}px`)
              .style('top', `${event.pageY - 10}px`)
              .html(`
                <div class="${styles.tooltipContent}">
                  <div class="${styles.tooltipTitle}">${metric.label}</div>
                  <div class="${styles.tooltipValue}">${Math.round(d[metric.key as keyof ChartData] as number)}%</div>
                  <div class="${styles.tooltipTime}">${d.timestamp.toLocaleTimeString()}</div>
                </div>
              `);
          })
          .on('mouseout', function() {
            d3.select(this).attr('opacity', 0);
            tooltip.style('opacity', 0);
          });
      });
    } else if (type === 'bar') {
      // Bar chart - show latest values
      const latest = data[data.length - 1];
      const barWidth = width / metrics.length;

      metrics.forEach((metric, index) => {
        const value = latest[metric.key as keyof ChartData] as number;
        const barHeight = height - yScale(value);

        svg.append('rect')
          .attr('x', index * barWidth + barWidth * 0.1)
          .attr('y', yScale(value))
          .attr('width', barWidth * 0.8)
          .attr('height', barHeight)
          .attr('fill', metric.color)
          .attr('rx', 4)
          .on('mouseover', function(event) {
            d3.select(this).attr('opacity', 0.8);

            tooltip
              .style('opacity', 1)
              .style('left', `${event.pageX + 10}px`)
              .style('top', `${event.pageY - 10}px`)
              .html(`
                <div class="${styles.tooltipContent}">
                  <div class="${styles.tooltipTitle}">${metric.label}</div>
                  <div class="${styles.tooltipValue}">${Math.round(value)}%</div>
                </div>
              `);
          })
          .on('mouseout', function() {
            d3.select(this).attr('opacity', 1);
            tooltip.style('opacity', 0);
          });
      });
    }

    // Legend
    const legend = svg.append('g')
      .attr('class', 'legend')
      .attr('transform', `translate(${width - 120}, 0)`);

    metrics.forEach((metric, index) => {
      const legendItem = legend.append('g')
        .attr('transform', `translate(0, ${index * 20})`);

      legendItem.append('rect')
        .attr('width', 12)
        .attr('height', 12)
        .attr('fill', metric.color)
        .attr('rx', 2);

      legendItem.append('text')
        .attr('x', 18)
        .attr('y', 9)
        .attr('font-size', '12px')
        .attr('fill', 'var(--color-text-primary)')
        .text(metric.label);
    });

  }, [data, type, timeRange]);

  if (!data.length) {
    return (
      <div className={styles.chartPlaceholder}>
        <Text variant="paragraph-medium" color="secondary">
          No data available for the selected time range
        </Text>
      </div>
    );
  }

  return (
    <div className={styles.chartWrapper}>
      <svg ref={svgRef} className={styles.chart}></svg>
      <div ref={tooltipRef} className={styles.tooltip}></div>
    </div>
  );
}
