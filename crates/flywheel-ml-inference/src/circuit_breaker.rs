use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,
    config: CircuitBreakerConfig,
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub half_open_max_calls: u32,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            half_open_max_calls: 3,
            reset_timeout: Duration::from_secs(30),
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            config,
        }
    }

    pub fn state(&self) -> CircuitState {
        *self.state.read()
    }

    pub fn can_execute(&self) -> bool {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = *self.last_failure_time.read() {
                    if last_failure.elapsed() > self.config.reset_timeout {
                        *self.state.write() = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold {
                    *self.state.write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&self) {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.failure_threshold {
                    *self.state.write() = CircuitState::Open;
                    *self.last_failure_time.write() = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                *self.state.write() = CircuitState::Open;
                *self.last_failure_time.write() = Some(Instant::now());
            }
            CircuitState::Open => {}
        }
    }
}
