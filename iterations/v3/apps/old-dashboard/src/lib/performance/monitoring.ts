/**
 * Performance Monitoring System
 * Web Vitals tracking and performance analytics
 */

import { onCLS, onINP, onFCP, onLCP, onTTFB } from 'web-vitals';

interface PerformanceMetrics {
  // Core Web Vitals
  cls: number | null;
  inp: number | null;
  lcp: number | null;
  
  // Additional metrics
  fcp: number | null;
  ttfb: number | null;
  
  // Custom metrics
  bundleSize: number;
  loadTime: number;
  renderTime: number;
  
  // User experience
  connectionType: string;
  deviceMemory: number;
  hardwareConcurrency: number;
}

interface PerformanceConfig {
  enabled: boolean;
  sampleRate: number;
  endpoint: string;
  debug: boolean;
}

class PerformanceMonitor {
  private config: PerformanceConfig;
  private metrics: Partial<PerformanceMetrics> = {};
  private observers: PerformanceObserver[] = [];

  constructor(config: Partial<PerformanceConfig> = {}) {
    this.config = {
      enabled: true,
      sampleRate: 1.0,
      endpoint: '/api/performance',
      debug: false,
      ...config,
    };

    if (this.config.enabled && this.shouldSample()) {
      this.initializeMonitoring();
    }
  }

  /**
   * Initialize performance monitoring
   */
  private initializeMonitoring(): void {
    this.setupWebVitals();
    this.setupCustomMetrics();
    this.setupResourceTiming();
    this.setupNavigationTiming();
  }

  /**
   * Setup Core Web Vitals monitoring
   */
  private setupWebVitals(): void {
    // Cumulative Layout Shift (CLS)
    onCLS((metric) => {
      this.metrics.cls = metric.value;
      this.logMetric('CLS', metric);
    });

    // Interaction to Next Paint (INP)
    onINP((metric) => {
      this.metrics.inp = metric.value;
      this.logMetric('INP', metric);
    });

    // Largest Contentful Paint (LCP)
    onLCP((metric) => {
      this.metrics.lcp = metric.value;
      this.logMetric('LCP', metric);
    });

    // First Contentful Paint (FCP)
    onFCP((metric) => {
      this.metrics.fcp = metric.value;
      this.logMetric('FCP', metric);
    });

    // Time to First Byte (TTFB)
    onTTFB((metric) => {
      this.metrics.ttfb = metric.value;
      this.logMetric('TTFB', metric);
    });
  }

  /**
   * Setup custom performance metrics
   */
  private setupCustomMetrics(): void {
    // Bundle size monitoring
    this.monitorBundleSize();
    
    // Load time monitoring
    this.monitorLoadTime();
    
    // Render time monitoring
    this.monitorLoadTime();
    
    // Device capabilities
    this.monitorDeviceCapabilities();
  }

  /**
   * Monitor bundle size
   */
  private monitorBundleSize(): void {
    if ('performance' in window && 'getEntriesByType' in performance) {
      const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[];
      const jsResources = resources.filter(resource => 
        resource.name.includes('.js') && 
        !resource.name.includes('node_modules')
      );
      
      const totalSize = jsResources.reduce((total, resource) => {
        return total + (resource.transferSize || 0);
      }, 0);
      
      this.metrics.bundleSize = totalSize;
    }
  }

  /**
   * Monitor page load time
   */
  private monitorLoadTime(): void {
    if ('performance' in window && 'getEntriesByType' in performance) {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      
      if (navigation) {
        this.metrics.loadTime = navigation.loadEventEnd - navigation.fetchStart;
        this.metrics.renderTime = navigation.domContentLoadedEventEnd - navigation.fetchStart;
      }
    }
  }

  /**
   * Monitor device capabilities
   */
  private monitorDeviceCapabilities(): void {
    // Connection type
    if ('connection' in navigator) {
      const connection = (navigator as any).connection;
      this.metrics.connectionType = connection?.effectiveType || 'unknown';
    }

    // Device memory
    if ('deviceMemory' in navigator) {
      this.metrics.deviceMemory = (navigator as any).deviceMemory || 0;
    }

    // Hardware concurrency
    this.metrics.hardwareConcurrency = navigator.hardwareConcurrency || 0;
  }

  /**
   * Setup resource timing monitoring
   */
  private setupResourceTiming(): void {
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        entries.forEach((entry) => {
          if (entry.entryType === 'resource') {
            this.analyzeResourceTiming(entry as PerformanceResourceTiming);
          }
        });
      });

      observer.observe({ entryTypes: ['resource'] });
      this.observers.push(observer);
    }
  }

  /**
   * Setup navigation timing monitoring
   */
  private setupNavigationTiming(): void {
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        entries.forEach((entry) => {
          if (entry.entryType === 'navigation') {
            this.analyzeNavigationTiming(entry as PerformanceNavigationTiming);
          }
        });
      });

      observer.observe({ entryTypes: ['navigation'] });
      this.observers.push(observer);
    }
  }

  /**
   * Analyze resource timing
   */
  private analyzeResourceTiming(entry: PerformanceResourceTiming): void {
    const loadTime = entry.responseEnd - entry.requestStart;
    const size = entry.transferSize || 0;
    
    // Log slow resources
    if (loadTime > 1000) {
      this.log(`Slow resource: ${entry.name} (${loadTime}ms, ${size}bytes)`);
    }
    
    // Log large resources
    if (size > 100000) {
      this.log(`Large resource: ${entry.name} (${size}bytes)`);
    }
  }

  /**
   * Analyze navigation timing
   */
  private analyzeNavigationTiming(entry: PerformanceNavigationTiming): void {
    const timing = {
      dns: entry.domainLookupEnd - entry.domainLookupStart,
      tcp: entry.connectEnd - entry.connectStart,
      request: entry.responseStart - entry.requestStart,
      response: entry.responseEnd - entry.responseStart,
      dom: entry.domContentLoadedEventEnd - entry.responseEnd,
    };

    this.log('Navigation timing:', timing);
  }

  /**
   * Log metric data
   */
  private logMetric(name: string, metric: any): void {
    if (this.config.debug) {
      console.log(`Performance Metric - ${name}:`, metric);
    }
    
    // Send to analytics endpoint
    this.sendMetric(name, metric);
  }

  /**
   * Send metric to analytics endpoint
   */
  private async sendMetric(name: string, metric: any): Promise<void> {
    try {
      await fetch(this.config.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          value: metric.value,
          delta: metric.delta,
          id: metric.id,
          navigationType: metric.navigationType,
          timestamp: Date.now(),
          url: window.location.href,
          userAgent: navigator.userAgent,
        }),
      });
    } catch (error) {
      console.error('Failed to send performance metric:', error);
    }
  }

  /**
   * Get current performance metrics
   */
  getMetrics(): Partial<PerformanceMetrics> {
    return { ...this.metrics };
  }

  /**
   * Check if we should sample this session
   */
  private shouldSample(): boolean {
    return Math.random() < this.config.sampleRate;
  }

  /**
   * Log debug information
   */
  private log(message: string, data?: any): void {
    if (this.config.debug) {
      console.log(`[Performance Monitor] ${message}`, data);
    }
  }

  /**
   * Cleanup observers
   */
  destroy(): void {
    this.observers.forEach(observer => observer.disconnect());
    this.observers = [];
  }
}

// Export singleton instance
export const performanceMonitor = new PerformanceMonitor({
  enabled: process.env.NODE_ENV === 'production',
  sampleRate: 0.1, // Sample 10% of users
  debug: process.env.NODE_ENV === 'development',
});

// Export for manual initialization
export { PerformanceMonitor };

// Performance utilities
export const performanceUtils = {
  /**
   * Measure function execution time
   */
  measure<T>(name: string, fn: () => T): T {
    const start = performance.now();
    const result = fn();
    const end = performance.now();
    
    console.log(`${name} took ${end - start} milliseconds`);
    return result;
  },

  /**
   * Measure async function execution time
   */
  async measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    const result = await fn();
    const end = performance.now();
    
    console.log(`${name} took ${end - start} milliseconds`);
    return result;
  },

  /**
   * Create performance mark
   */
  mark(name: string): void {
    if ('performance' in window && 'mark' in performance) {
      performance.mark(name);
    }
  },

  /**
   * Measure between two marks
   */
  measureMarks(name: string, startMark: string, endMark: string): void {
    if ('performance' in window && 'measure' in performance) {
      try {
        performance.measure(name, startMark, endMark);
      } catch (error) {
        console.warn(`Failed to measure ${name}:`, error);
      }
    }
  },
};
