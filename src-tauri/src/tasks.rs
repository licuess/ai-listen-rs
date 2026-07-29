use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize)]
pub struct TaskStatus {
    pub id: String,
    pub kind: String,
    pub state: String,
    pub message: String,
    pub progress: u8,
    pub partial: String,
    pub session_slug: Option<String>,
}

static TASKS: OnceLock<Mutex<BTreeMap<String, TaskStatus>>> = OnceLock::new();

pub fn create(kind: &str, session_slug: Option<String>, message: &str) -> TaskStatus {
    let task = TaskStatus {
        id: timestamp(),
        kind: kind.to_string(),
        state: "queued".to_string(),
        message: message.to_string(),
        progress: 0,
        partial: String::new(),
        session_slug,
    };
    upsert(task.clone());
    task
}

pub fn update_progress(
    id: &str,
    state: &str,
    message: &str,
    progress: Option<u8>,
    partial: Option<&str>,
) {
    let mut tasks = storage().lock().expect("task storage poisoned");
    if let Some(task) = tasks.get_mut(id) {
        task.state = state.to_string();
        task.message = message.to_string();
        if let Some(progress) = progress {
            task.progress = progress.min(100);
        }
        if let Some(partial) = partial {
            task.partial = partial.to_string();
        }
    }
}

pub fn append_partial(id: &str, chunk_text: &str) {
    let mut tasks = storage().lock().expect("task storage poisoned");
    if let Some(task) = tasks.get_mut(id) {
        if !task.partial.is_empty() && !chunk_text.is_empty() {
            task.partial.push_str("\n\n");
        }
        task.partial.push_str(chunk_text);
    }
}

pub fn get(id: &str) -> Option<TaskStatus> {
    storage()
        .lock()
        .expect("task storage poisoned")
        .get(id)
        .cloned()
}

pub fn list() -> Vec<TaskStatus> {
    storage()
        .lock()
        .expect("task storage poisoned")
        .values()
        .cloned()
        .collect()
}

fn upsert(task: TaskStatus) {
    storage()
        .lock()
        .expect("task storage poisoned")
        .insert(task.id.clone(), task);
}

fn storage() -> &'static Mutex<BTreeMap<String, TaskStatus>> {
    TASKS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("task-{nanos}")
}
