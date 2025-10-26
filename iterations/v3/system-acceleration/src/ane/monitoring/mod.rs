//! YOLO Performance Monitoring Module
//!
//! Comprehensive monitoring and optimization for YOLO inference performance.

pub mod yolo_monitor;
pub mod dashboard;

// Re-export main types
pub use yolo_monitor::{
    YOLOPerformanceMonitor, YOLOPerformanceMetrics, YOLOPerformanceStats,
    YOLOPerformanceThresholds, create_yolo_metrics
};

pub use dashboard::{
    YOLOPerformanceDashboard, DashboardMetrics, PerformanceAlerts,
    PerformanceAlert, AlertLevel, PerformancePredictor, PerformancePrediction,
    PerformanceTrend
};
