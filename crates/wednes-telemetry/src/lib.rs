use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Telemetry {
    pub active_invocations: Arc<AtomicUsize>,
    pub total_invocations: Arc<AtomicUsize>,
    pub traps: Arc<AtomicUsize>,
    pub rejected: Arc<AtomicUsize>,
    pub total_duration_ms: Arc<AtomicU64>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            active_invocations: Arc::new(AtomicUsize::new(0)),
            total_invocations: Arc::new(AtomicUsize::new(0)),
            traps: Arc::new(AtomicUsize::new(0)),
            rejected: Arc::new(AtomicUsize::new(0)),
            total_duration_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_start(&self) {
        self.total_invocations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_reject(&self, id: &str, start: Instant) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
        let duration = start.elapsed().as_millis();
        self.total_duration_ms
            .fetch_add(duration as u64, Ordering::Relaxed);
        info!(
            id = %id,
            status = 429,
            duration_ms = duration,
            traps = false,
            reject = true,
            active = self.active_invocations.load(Ordering::Relaxed),
            "invocation rejected"
        );
    }

    pub fn record_accept(&self) {
        self.active_invocations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_complete(&self, id: &str, status: u16, start: Instant) {
        self.active_invocations.fetch_sub(1, Ordering::Relaxed);
        let duration = start.elapsed().as_millis();
        self.total_duration_ms
            .fetch_add(duration as u64, Ordering::Relaxed);
        info!(
            id = %id,
            status = status,
            duration_ms = duration,
            traps = false,
            reject = false,
            active = self.active_invocations.load(Ordering::Relaxed),
            "invocation completed"
        );
    }

    pub fn record_trap(&self, id: &str, err: impl std::fmt::Display, start: Instant) {
        self.active_invocations.fetch_sub(1, Ordering::Relaxed);
        self.traps.fetch_add(1, Ordering::Relaxed);
        let duration = start.elapsed().as_millis();
        self.total_duration_ms
            .fetch_add(duration as u64, Ordering::Relaxed);
        info!(
            id = %id,
            status = 500,
            duration_ms = duration,
            traps = true,
            reject = false,
            error = %err,
            active = self.active_invocations.load(Ordering::Relaxed),
            "invocation trapped"
        );
    }

    pub fn prometheus_metrics(&self) -> String {
        let invs = self.total_invocations.load(Ordering::Relaxed);
        let dur = self.total_duration_ms.load(Ordering::Relaxed);
        let traps = self.traps.load(Ordering::Relaxed);
        let rejs = self.rejected.load(Ordering::Relaxed);
        let acts = self.active_invocations.load(Ordering::Relaxed);

        format!(
            "# HELP gateway_invocations_total Total number of function invocations\n\
             # TYPE gateway_invocations_total counter\n\
             gateway_invocations_total {}\n\
             # HELP gateway_duration_ms_total Total duration of invocations in ms\n\
             # TYPE gateway_duration_ms_total counter\n\
             gateway_duration_ms_total {}\n\
             # HELP gateway_traps_total Total number of trap errors\n\
             # TYPE gateway_traps_total counter\n\
             gateway_traps_total {}\n\
             # HELP gateway_rejected_total Total number of rejected invocations\n\
             # TYPE gateway_rejected_total counter\n\
             gateway_rejected_total {}\n\
             # HELP gateway_active_executions Number of currently executing functions\n\
             # TYPE gateway_active_executions gauge\n\
             gateway_active_executions {}\n",
            invs, dur, traps, rejs, acts
        )
    }
}
