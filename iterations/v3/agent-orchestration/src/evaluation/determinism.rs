//! Determinism and Reproducibility Harness
//!
//! Provides Clock and RngSource traits for deterministic, reproducible agent execution.

use chrono::{DateTime, Utc};
use rand::rngs::StdRng;

/// Clock trait for time abstraction
pub trait Clock: Send + Sync {
    /// Get current time
    fn now(&self) -> DateTime<Utc>;
}

/// System clock implementation (uses real time)
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Fixed clock implementation (for deterministic testing)
pub struct FixedClock {
    fixed_time: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self { fixed_time: time }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.fixed_time
    }
}

/// Random number generator source trait
pub trait RngSource: Send + Sync {
    /// Get mutable reference to RNG
    fn rng(&mut self) -> &mut StdRng;
}

/// Thread-safe wrapper for RNG source (for use with Arc)
pub struct ThreadSafeRngSource {
    rng: std::sync::Mutex<Box<dyn RngSource + Send>>,
}

impl ThreadSafeRngSource {
    pub fn new(source: Box<dyn RngSource + Send>) -> Self {
        Self {
            rng: std::sync::Mutex::new(source),
        }
    }

    /// Generate a UUID deterministically using the RNG
    pub fn generate_uuid(&self) -> uuid::Uuid {
        use rand::RngCore;
        use uuid::Uuid;
        let mut rng_guard = self.rng.lock().unwrap();
        let rng = rng_guard.rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        Uuid::from_bytes(bytes)
    }

    /// Generate a random u64
    pub fn next_u64(&self) -> u64 {
        use rand::RngCore;
        let mut rng_guard = self.rng.lock().unwrap();
        let rng = rng_guard.rng();
        rng.next_u64()
    }
}

/// Seeded RNG source for deterministic testing
pub struct SeededRng {
    rng: StdRng,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl RngSource for SeededRng {
    fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }
}

/// System RNG source (uses system randomness)
pub struct SystemRng {
    rng: StdRng,
}

impl SystemRng {
    pub fn new() -> Self {
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut os_rng = OsRng;
        let seed = os_rng.next_u64();
        use rand::SeedableRng;
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl RngSource for SystemRng {
    fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_clock() {
        let fixed_time = Utc::now();
        let clock = FixedClock::new(fixed_time);

        assert_eq!(clock.now(), fixed_time);
        assert_eq!(clock.now(), fixed_time); // Should be consistent
    }

    #[test]
    fn test_seeded_rng() {
        let mut rng1 = SeededRng::new(42);
        let mut rng2 = SeededRng::new(42);

        // Same seed should produce same sequence
        assert_eq!(rng1.rng().next_u32(), rng2.rng().next_u32());
    }

    #[test]
    fn test_thread_safe_rng_determinism() {
        use rand::RngCore;

        // Create two ThreadSafeRngSource instances with the same seed
        let rng1 = ThreadSafeRngSource::new(Box::new(SeededRng::new(42)));
        let rng2 = ThreadSafeRngSource::new(Box::new(SeededRng::new(42)));

        // Generate UUIDs - should be deterministic
        let uuid1 = rng1.generate_uuid();
        let uuid2 = rng2.generate_uuid();

        // Same seed should produce same UUID
        assert_eq!(uuid1, uuid2);

        // Generate more UUIDs to verify sequence
        let uuid3 = rng1.generate_uuid();
        let uuid4 = rng2.generate_uuid();
        assert_eq!(uuid3, uuid4);
    }

    #[test]
    fn test_thread_safe_rng_u64_determinism() {
        // Create two ThreadSafeRngSource instances with the same seed
        let rng1 = ThreadSafeRngSource::new(Box::new(SeededRng::new(123)));
        let rng2 = ThreadSafeRngSource::new(Box::new(SeededRng::new(123)));

        // Generate u64 values - should be deterministic
        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();

        // Same seed should produce same value
        assert_eq!(val1, val2);
    }
}
