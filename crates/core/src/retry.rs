use std::time::Duration;

use rand::{thread_rng, Rng};

pub fn backoff_delay(attempt: usize, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
    let exp = 2u64.saturating_pow(attempt as u32);
    let base = base_delay_ms.saturating_mul(exp).min(max_delay_ms);
    let jitter: u64 = thread_rng().gen_range(0..=base_delay_ms);
    Duration::from_millis(base.saturating_add(jitter))
}

pub async fn sleep_with_jitter(attempt: usize, base_delay_ms: u64, max_delay_ms: u64) {
    tokio::time::sleep(backoff_delay(attempt, base_delay_ms, max_delay_ms)).await;
}
