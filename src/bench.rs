use std::sync::OnceLock;
use std::time::Instant;

static PROCESS_START: OnceLock<Instant> = OnceLock::new();

pub fn set_process_start(t: Instant) {
    let _ = PROCESS_START.set(t);
}

pub fn since_process_start() -> Option<std::time::Duration> {
    PROCESS_START.get().map(|t| t.elapsed())
}
