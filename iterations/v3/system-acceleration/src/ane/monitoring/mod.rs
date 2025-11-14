//! YOLO Performance Monitoring Module
//!
//! Comprehensive monitoring and optimization for YOLO inference performance.

pub mod dashboard;
pub mod yolo_monitor;

// Re-export main types
pub use yolo_monitor::{
    create_yolo_metrics, YOLOPerformanceMetrics, YOLOPerformanceMonitor, YOLOPerformanceStats,
    YOLOPerformanceThresholds,
};

pub use dashboard::{
    AlertLevel, DashboardMetrics, PerformanceAlert, PerformanceAlerts, PerformancePrediction,
    PerformancePredictor, PerformanceTrend, YOLOPerformanceDashboard,
};
