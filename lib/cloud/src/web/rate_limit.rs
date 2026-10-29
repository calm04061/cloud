use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_FAILURES: u32 = 5;
const WINDOW: Duration = Duration::from_secs(15 * 60);
const MAX_TRACKED_KEYS: usize = 10_000;

#[derive(Debug, Clone, Copy)]
struct Attempt {
    count: u32,
    last: Instant,
}

type Attempts = Mutex<HashMap<String, Attempt>>;

static ATTEMPTS: OnceLock<Attempts> = OnceLock::new();

fn attempts() -> &'static Attempts {
    ATTEMPTS.get_or_init(Default::default)
}

pub(crate) fn allowed(key: &str) -> bool {
    let attempts = attempts().lock().expect("auth attempts lock poisoned");
    match attempts.get(key) {
        Some(attempt) => {
            attempt.count < MAX_FAILURES || attempt.last.elapsed() >= WINDOW
        }
        None => true,
    }
}

pub(crate) fn failure(key: &str) {
    let mut attempts = attempts().lock().expect("auth attempts lock poisoned");
    if attempts.len() >= MAX_TRACKED_KEYS {
        attempts.retain(|_, attempt| attempt.last.elapsed() < WINDOW);
    }
    let attempt = attempts.entry(key.to_string()).or_insert(Attempt {
        count: 0,
        last: Instant::now(),
    });
    if attempt.last.elapsed() >= WINDOW {
        attempt.count = 0;
    }
    attempt.count += 1;
    attempt.last = Instant::now();
}

pub(crate) fn success(key: &str) {
    let mut attempts = attempts().lock().expect("auth attempts lock poisoned");
    attempts.remove(key);
}
