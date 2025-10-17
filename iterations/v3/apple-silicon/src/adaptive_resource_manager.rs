use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeviceKind { Cpu, Gpu, Ane }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default)]
pub struct StaticModelRegistry {
    pub entries: std::collections::HashMap<String, std::collections::HashMap<DeviceKind, Vec<Precision>>>,
}

impl StaticModelRegistry {
    pub fn with_entry(mut self, model: &str, device: DeviceKind, precs: Vec<Precision>) -> Self {
        let devs = self.entries.entry(model.to_string()).or_default();
        devs.insert(device, precs);
        self
    }
}

impl ModelRegistry for StaticModelRegistry {
    fn supports(&self, model: &str, device: DeviceKind, precision: Precision) -> bool {
        self.entries
            .get(model)
            .and_then(|m| m.get(&device))
            .map(|v| v.contains(&precision))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AppleModelRegistryConfig {
    pub models: std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Default)]
pub struct AppleModelRegistry { inner: StaticModelRegistry }

impl AppleModelRegistry {
    pub fn from_config(cfg: AppleModelRegistryConfig) -> Self {
        let mut reg = StaticModelRegistry::default();
        for (model, devs) in cfg.models {
            for (dev_s, precs_s) in devs {
                let device = match dev_s.as_str() {
                    "Ane"|"ANE"|"ane" => DeviceKind::Ane,
                    "Gpu"|"GPU"|"gpu" => DeviceKind::Gpu,
                    _ => DeviceKind::Cpu,
                };
                let precs = precs_s.into_iter().filter_map(|p| match p.as_str() {
                    "Int4"|"INT4"|"int4" => Some(Precision::Int4),
                    "Int8"|"INT8"|"int8" => Some(Precision::Int8),
                    "Fp16"|"FP16"|"fp16" => Some(Precision::Fp16),
                    "Fp32"|"FP32"|"fp32" => Some(Precision::Fp32),
                    _ => None,
                }).collect::<Vec<_>>();
                if !precs.is_empty() { reg = reg.with_entry(&model, device, precs); }
            }
        }
        Self { inner: reg }
    }

    pub fn from_json_str(s: &str) -> Option<Self> {
        serde_json::from_str::<AppleModelRegistryConfig>(s).ok().map(Self::from_config)
    }

    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        std::fs::read_to_string(path).ok().and_then(|c| Self::from_json_str(&c))
    }
}

impl ModelRegistry for AppleModelRegistry {
    fn supports(&self, model: &str, device: DeviceKind, precision: Precision) -> bool {
        self.inner.supports(model, device, precision)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StaticSensors {
    pub ane: ThermalState,
    pub gpu: ThermalState,
    pub cpu: ThermalState,
    pub mem: MemoryState,
}

impl DeviceSensors for StaticSensors {
    fn thermal(&self, device: DeviceKind) -> ThermalState {
        match device {
            DeviceKind::Ane => self.ane,
            DeviceKind::Gpu => self.gpu,
            DeviceKind::Cpu => self.cpu,
        }
    }
    fn memory(&self) -> MemoryState { self.mem }
}

/// Heuristic system sensors backed by OS where available. Safe fallbacks otherwise.
pub struct SystemSensors {
    ane_env: Option<bool>,
    gpu_env: Option<bool>,
    cpu_env: Option<bool>,
    ane_head: Option<u8>,
    gpu_head: Option<u8>,
    cpu_head: Option<u8>,
}

impl SystemSensors {
    pub fn detect() -> Self {
        Self {
            ane_env: std::env::var("ARM_FORCE_THROTTLE_ANE").ok().and_then(|v| v.parse().ok()),
            gpu_env: std::env::var("ARM_FORCE_THROTTLE_GPU").ok().and_then(|v| v.parse().ok()),
            cpu_env: std::env::var("ARM_FORCE_THROTTLE_CPU").ok().and_then(|v| v.parse().ok()),
            ane_head: std::env::var("ARM_HEADROOM_ANE").ok().and_then(|v| v.parse().ok()),
            gpu_head: std::env::var("ARM_HEADROOM_GPU").ok().and_then(|v| v.parse().ok()),
            cpu_head: std::env::var("ARM_HEADROOM_CPU").ok().and_then(|v| v.parse().ok()),
        }
    }

    fn macos_memory_state() -> Option<MemoryState> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let out = Command::new("vm_stat").output().ok()?;
            let s = String::from_utf8_lossy(&out.stdout);
            let mut page_size = 4096.0;
            let mut free = 0.0;
            let mut active = 0.0;
            let mut wired = 0.0;
            for line in s.lines() {
                if line.contains("page size of") {
                    if let Some(ps) = line.split_whitespace().filter_map(|t| t.replace(",","" ).parse::<f64>().ok()).last() { page_size = ps; }
                } else if line.starts_with("Pages free:") {
                    free = line.split_whitespace().filter_map(|t| t.replace(",","" ).parse::<f64>().ok()).last().unwrap_or(0.0);
                } else if line.starts_with("Pages active:") {
                    active = line.split_whitespace().filter_map(|t| t.replace(",","" ).parse::<f64>().ok()).last().unwrap_or(0.0);
                } else if line.starts_with("Pages wired down:") {
                    wired = line.split_whitespace().filter_map(|t| t.replace(",","" ).parse::<f64>().ok()).last().unwrap_or(0.0);
                }
            }
            let used_bytes = (active + wired) * page_size;
            // Get total via sysctl hw.memsize
            let total_bytes = Command::new("sysctl").arg("-n").arg("hw.memsize").output().ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(32.0 * 1024.0 * 1024.0 * 1024.0);
            return Some(MemoryState { used_gb: (used_bytes / (1024.0*1024.0*1024.0)) as f32, total_gb: (total_bytes / (1024.0*1024.0*1024.0)) as f32 });
        }
        #[allow(unreachable_code)] None
    }

    fn default_headroom(mem: &MemoryState) -> u8 {
        let usage = mem.used_gb / mem.total_gb;
        if usage > 0.9 { 10 } else if usage > 0.8 { 20 } else if usage > 0.7 { 30 } else { 60 }
    }
}

impl DeviceSensors for SystemSensors {
    fn thermal(&self, device: DeviceKind) -> ThermalState {
        let mem = self.memory();
        let head_env = match device {
            DeviceKind::Ane => self.ane_head,
            DeviceKind::Gpu => self.gpu_head,
            DeviceKind::Cpu => self.cpu_head,
        };
        let thr_env = match device {
            DeviceKind::Ane => self.ane_env,
            DeviceKind::Gpu => self.gpu_env,
            DeviceKind::Cpu => self.cpu_env,
        };
        let head = head_env.unwrap_or_else(|| Self::default_headroom(&mem));
        let throttled = thr_env.unwrap_or(false) || head < 15;
        ThermalState { throttled, headroom_pct: head }
    }
    fn memory(&self) -> MemoryState {
        Self::macos_memory_state().unwrap_or(MemoryState { used_gb: 8.0, total_gb: 32.0 })
    }
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

    fn open_registry() -> StaticModelRegistry {
        StaticModelRegistry::default()
            .with_entry("judge", DeviceKind::Ane, vec![Precision::Int8, Precision::Fp16])
            .with_entry("judge", DeviceKind::Gpu, vec![Precision::Fp16, Precision::Int8])
            .with_entry("worker", DeviceKind::Gpu, vec![Precision::Int4, Precision::Int8, Precision::Fp16])
            .with_entry("worker", DeviceKind::Cpu, vec![Precision::Fp32])
    }

    #[test]
    fn prefers_ane_when_not_throttled() {
        let sensors = StaticSensors{ ane: ThermalState{ throttled: false, headroom_pct: 80}, gpu: ThermalState{ throttled: false, headroom_pct: 70}, cpu: ThermalState{ throttled: false, headroom_pct: 90}, mem: MemoryState{ used_gb: 8.0, total_gb: 32.0 } };
        let planner = SimplePlanner::new(sensors, open_registry());
        let req = AllocationRequest { model: "judge".into(), supported_precisions: vec![Precision::Int8, Precision::Fp16], preferred_devices: vec![], tier: Tier::T1, latency_slo_ms: 20, max_batch_size: 8, workload_hint: WorkloadHint::JudgeLatencySensitive };
        let plan = planner.plan(&req);
        assert_eq!(plan.device, DeviceKind::Ane);
        assert!(plan.batch_size >= 1);
    }

    #[test]
    fn falls_back_when_throttled() {
        let sensors = StaticSensors{ ane: ThermalState{ throttled: true, headroom_pct: 5}, gpu: ThermalState{ throttled: false, headroom_pct: 60}, cpu: ThermalState{ throttled: false, headroom_pct: 90}, mem: MemoryState{ used_gb: 8.0, total_gb: 32.0 } };
        let planner = SimplePlanner::new(sensors, open_registry());
        let req = AllocationRequest { model: "judge".into(), supported_precisions: vec![Precision::Int8, Precision::Fp16], preferred_devices: vec![DeviceKind::Ane], tier: Tier::T1, latency_slo_ms: 15, max_batch_size: 8, workload_hint: WorkloadHint::JudgeLatencySensitive };
        let plan = planner.plan(&req);
        assert!(plan.device == DeviceKind::Gpu || plan.device == DeviceKind::Cpu);
    }

    #[test]
    fn throughput_prefers_low_precision() {
        let sensors = StaticSensors{ ane: ThermalState{ throttled: false, headroom_pct: 80}, gpu: ThermalState{ throttled: false, headroom_pct: 60}, cpu: ThermalState{ throttled: false, headroom_pct: 90}, mem: MemoryState{ used_gb: 8.0, total_gb: 32.0 } };
        let planner = SimplePlanner::new(sensors, open_registry());
        let req = AllocationRequest { model: "worker".into(), supported_precisions: vec![Precision::Int4, Precision::Int8, Precision::Fp16], preferred_devices: vec![DeviceKind::Gpu], tier: Tier::T2, latency_slo_ms: 100, max_batch_size: 32, workload_hint: WorkloadHint::WorkerThroughput };
        let plan = planner.plan(&req);
        assert_eq!(plan.precision, Precision::Int4);
        assert!(plan.batch_size >= 1);
    }
}
