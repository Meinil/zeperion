use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU8, Ordering},
    },
    time::Instant,
};

const RUNNING: u8 = 0;
const PAUSE: u8 = 1;
const CANCEL: u8 = 2;

fn registry() -> &'static Mutex<HashMap<String, Arc<AtomicU8>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Arc<AtomicU8>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn metrics_registry() -> &'static Mutex<HashMap<String, TaskMetrics>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, TaskMetrics>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Debug, Default)]
pub struct TaskRuntimeSnapshot {
    pub current_target: Option<String>,
    pub speed_bps: Option<f64>,
    pub sections: Vec<SectionRuntimeSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct SectionRuntimeSnapshot {
    pub key: String,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub completed_items: i64,
    pub total_items: i64,
    pub state: String,
}

#[derive(Debug, Default)]
struct TaskMetrics {
    current_target: Option<String>,
    speed_bps: Option<f64>,
    last_total_bytes: Option<i64>,
    last_sample_at: Option<Instant>,
    last_emit_at: Option<Instant>,
    sections: HashMap<String, SectionRuntimeSnapshot>,
}

pub fn register(task_id: String) -> Arc<AtomicU8> {
    let control = Arc::new(AtomicU8::new(RUNNING));
    if let Ok(mut map) = registry().lock() {
        map.insert(task_id.clone(), control.clone());
    }
    if let Ok(mut map) = metrics_registry().lock() {
        map.insert(task_id, TaskMetrics::default());
    }
    control
}

pub fn unregister(task_id: &str) {
    if let Ok(mut map) = registry().lock() {
        map.remove(task_id);
    }
    if let Ok(mut map) = metrics_registry().lock() {
        map.remove(task_id);
    }
}

pub fn request_pause(task_id: &str) -> bool {
    request(task_id, PAUSE)
}

pub fn request_cancel(task_id: &str) -> bool {
    request(task_id, CANCEL)
}

pub fn is_running(task_id: &str) -> bool {
    registry()
        .lock()
        .ok()
        .and_then(|map| {
            map.get(task_id)
                .map(|state| state.load(Ordering::SeqCst) == RUNNING)
        })
        .unwrap_or(false)
}

pub fn is_pause_requested(control: &Arc<AtomicU8>) -> bool {
    control.load(Ordering::SeqCst) == PAUSE
}

pub fn is_cancel_requested(control: &Arc<AtomicU8>) -> bool {
    control.load(Ordering::SeqCst) == CANCEL
}

pub fn set_current_target(task_id: &str, target: impl Into<String>) {
    if let Ok(mut map) = metrics_registry().lock() {
        if let Some(metrics) = map.get_mut(task_id) {
            metrics.current_target = Some(target.into());
        }
    }
}

pub fn record_progress(task_id: &str, total_downloaded_bytes: i64) {
    let now = Instant::now();
    if let Ok(mut map) = metrics_registry().lock() {
        if let Some(metrics) = map.get_mut(task_id) {
            if let (Some(last_bytes), Some(last_at)) =
                (metrics.last_total_bytes, metrics.last_sample_at)
            {
                let elapsed = now.duration_since(last_at).as_secs_f64();
                let delta = total_downloaded_bytes.saturating_sub(last_bytes) as f64;
                if elapsed > 0.15 && delta >= 0.0 {
                    metrics.speed_bps = Some(delta / elapsed);
                    metrics.last_total_bytes = Some(total_downloaded_bytes);
                    metrics.last_sample_at = Some(now);
                    return;
                }
            }

            if metrics.last_total_bytes.is_none() || metrics.last_sample_at.is_none() {
                metrics.last_total_bytes = Some(total_downloaded_bytes);
                metrics.last_sample_at = Some(now);
                metrics.speed_bps = Some(0.0);
            }
        }
    }
}

pub fn should_emit_progress(task_id: &str, force: bool) -> bool {
    if let Ok(mut map) = metrics_registry().lock() {
        if let Some(metrics) = map.get_mut(task_id) {
            let now = Instant::now();
            if force {
                metrics.last_emit_at = Some(now);
                return true;
            }

            match metrics.last_emit_at {
                Some(last_at) if now.duration_since(last_at).as_secs_f64() < 0.12 => false,
                _ => {
                    metrics.last_emit_at = Some(now);
                    true
                }
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn set_section_snapshot(task_id: &str, section: SectionRuntimeSnapshot) {
    if let Ok(mut map) = metrics_registry().lock() {
        if let Some(metrics) = map.get_mut(task_id) {
            metrics.sections.insert(section.key.clone(), section);
        }
    }
}

pub fn snapshot(task_id: &str) -> Option<TaskRuntimeSnapshot> {
    metrics_registry().lock().ok().and_then(|map| {
        map.get(task_id).map(|metrics| TaskRuntimeSnapshot {
            current_target: metrics.current_target.clone(),
            speed_bps: metrics.speed_bps,
            sections: metrics.sections.values().cloned().collect(),
        })
    })
}

fn request(task_id: &str, value: u8) -> bool {
    if let Ok(map) = registry().lock() {
        if let Some(control) = map.get(task_id) {
            control.store(value, Ordering::SeqCst);
            return true;
        }
    }
    false
}
