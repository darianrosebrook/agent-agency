use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceKind { Cpu, Gpu, Ane }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Precision { Int4, Int8, Fp16, Fp32 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tier { T1, T2, T3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub model: String,
    pub supported_precisions: Vec<Precision>,
    pub preferred_devices: Vec<DeviceKind>,
    pub tier: Tier,
    pub latency_slo_ms: u32,
    pub max_batch_size: u32,
    pub workload_hint: WorkloadHint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WorkloadHint { JudgeLatencySensitive, WorkerThroughput }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPlan {
    pub device: DeviceKind,
    pub precision: Precision,
    pub batch_size: u32,
    pub expected_latency_ms: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThermalState { pub throttled: bool, pub headroom_pct: u8 }

#[derive(Debug, Clone, Copy)]
pub struct MemoryState { pub used_gb: f32, pub total_gb: f32 }

pub trait DeviceSensors: Send + Sync {
    fn thermal(&self, device: DeviceKind) -> ThermalState;
    fn memory(&self) -> MemoryState;
}

pub trait ModelRegistry: Send + Sync {
    fn supports(&self, model: &str, device: DeviceKind, precision: Precision) -> bool;
}

pub trait AllocationPlanner: Send + Sync {
    fn plan(&self, req: &AllocationRequest) -> AllocationPlan;
}

pub struct SimplePlanner<S: DeviceSensors, R: ModelRegistry> { sensors: S, registry: R }

impl<S: DeviceSensors, R: ModelRegistry> SimplePlanner<S, R> {
    pub fn new(sensors: S, registry: R) -> Self { Self { sensors, registry } }

    fn choose_device(&self, req: &AllocationRequest) -> DeviceKind {
        // Prefer preferred_devices if supported and not throttled; fallback ANE→GPU→CPU.
        let candidates = if req.preferred_devices.is_empty() {
            vec![DeviceKind::Ane, DeviceKind::Gpu, DeviceKind::Cpu]
        } else { req.preferred_devices.clone() };

        for d in candidates {
            let thermal = self.sensors.thermal(d);
            if thermal.throttled { continue; }
            // choose first supported precision on device
            if req.supported_precisions.iter().any(|&p| self.registry.supports(&req.model, d, p)) {
                return d;
            }
        }
        // fallback: pick first supported ignoring throttle
        for d in [DeviceKind::Ane, DeviceKind::Gpu, DeviceKind::Cpu] {
            if req.supported_precisions.iter().any(|&p| self.registry.supports(&req.model, d, p)) {
                return d;
            }
        }
        DeviceKind::Cpu
    }

    fn choose_precision(&self, req: &AllocationRequest, device: DeviceKind) -> Precision {
        // favor lower precision for throughput if supported; higher for quality if judge
        let mut prefs = match req.workload_hint {
            WorkloadHint::WorkerThroughput => [Precision::Int4, Precision::Int8, Precision::Fp16, Precision::Fp32],
            WorkloadHint::JudgeLatencySensitive => [Precision::Int8, Precision::Fp16, Precision::Fp32, Precision::Int4],
        };
        for &p in &prefs { if req.supported_precisions.contains(&p) && self.registry.supports(&req.model, device, p) { return p; } }
        // fallback to any supported
        req.supported_precisions.first().copied().unwrap_or(Precision::Fp16)
    }

    fn estimate_latency_ms(&self, device: DeviceKind, precision: Precision, batch: u32) -> u32 {
        // very rough heuristic for initial policy tests
        let base = match (device, precision) {
            (DeviceKind::Ane, Precision::Int4) => 5,
            (DeviceKind::Ane, Precision::Int8) => 7,
            (DeviceKind::Gpu, Precision::Fp16) => 12,
            (DeviceKind::Gpu, Precision::Int8) => 10,
            (DeviceKind::Cpu, Precision::Fp32) => 40,
            _ => 20,
        } as u32;
        base.saturating_add(batch.saturating_mul(2))
    }
}

impl<S: DeviceSensors, R: ModelRegistry> AllocationPlanner for SimplePlanner<S, R> {
    fn plan(&self, req: &AllocationRequest) -> AllocationPlan {
        let mut device = self.choose_device(req);
        let mut precision = self.choose_precision(req, device);
        let mut batch = req.max_batch_size.max(1);

        // Throttle-aware derating
        let thermal = self.sensors.thermal(device);
        if thermal.throttled { batch = (batch / 2).max(1); }

        // SLO-aware controller
        let mut est = self.estimate_latency_ms(device, precision, batch);
        if let Tier::T1 = req.tier {
            while est > req.latency_slo_ms && batch > 1 { batch = (batch + 1) / 2; est = self.estimate_latency_ms(device, precision, batch); }
            if est > req.latency_slo_ms {
                // fallback to CPU if still missing SLO to avoid thermal constraints
                device = DeviceKind::Cpu;
                precision = self.choose_precision(req, device);
                est = self.estimate_latency_ms(device, precision, batch);
            }
        }

        AllocationPlan { device, precision, batch_size: batch, expected_latency_ms: est, notes: None }
    }
}

// -------------------- Tests --------------------
#[cfg(test)]
mod tests {
    use super::*;

    struct MockSensors { pub ane_throttled: bool }
    impl DeviceSensors for MockSensors {
        fn thermal(&self, d: DeviceKind) -> ThermalState { match d { DeviceKind::Ane => ThermalState{ throttled: self.ane_throttled, headroom_pct: 10 }, _ => ThermalState{ throttled: false, headroom_pct: 60 } } }
        fn memory(&self) -> MemoryState { MemoryState{ used_gb: 8.0, total_gb: 32.0 } }
    }
    struct MockRegistry;
    impl ModelRegistry for MockRegistry {
        fn supports(&self, _model: &str, _device: DeviceKind, _precision: Precision) -> bool { true }
    }

    #[test]
    fn prefers_ane_when_not_throttled() {
        let planner = SimplePlanner::new(MockSensors{ ane_throttled: false }, MockRegistry);
        let req = AllocationRequest { model: "judge".into(), supported_precisions: vec![Precision::Int8, Precision::Fp16], preferred_devices: vec![], tier: Tier::T1, latency_slo_ms: 20, max_batch_size: 8, workload_hint: WorkloadHint::JudgeLatencySensitive };
        let plan = planner.plan(&req);
        assert_eq!(plan.device, DeviceKind::Ane);
        assert!(plan.batch_size >= 1);
    }

    #[test]
    fn falls_back_when_throttled() {
        let planner = SimplePlanner::new(MockSensors{ ane_throttled: true }, MockRegistry);
        let req = AllocationRequest { model: "judge".into(), supported_precisions: vec![Precision::Int8, Precision::Fp16], preferred_devices: vec![DeviceKind::Ane], tier: Tier::T1, latency_slo_ms: 15, max_batch_size: 8, workload_hint: WorkloadHint::JudgeLatencySensitive };
        let plan = planner.plan(&req);
        assert!(plan.device == DeviceKind::Gpu || plan.device == DeviceKind::Cpu);
    }

    #[test]
    fn throughput_prefers_low_precision() {
        let planner = SimplePlanner::new(MockSensors{ ane_throttled: false }, MockRegistry);
        let req = AllocationRequest { model: "worker".into(), supported_precisions: vec![Precision::Int4, Precision::Int8, Precision::Fp16], preferred_devices: vec![DeviceKind::Gpu], tier: Tier::T2, latency_slo_ms: 100, max_batch_size: 32, workload_hint: WorkloadHint::WorkerThroughput };
        let plan = planner.plan(&req);
        assert_eq!(plan.precision, Precision::Int4);
        assert!(plan.batch_size >= 1);
    }
}

