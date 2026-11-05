use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use serde::Serialize;

#[derive(Default)]
pub struct ActiveJobs {
    jobs: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl ActiveJobs {
    pub fn register(&self, id: String) -> Arc<AtomicBool> {
        let cancelled = Arc::new(AtomicBool::new(false));
        self.jobs
            .lock()
            .expect("active jobs lock")
            .insert(id, cancelled.clone());
        cancelled
    }

    pub fn cancel(&self, id: &str) -> bool {
        if let Some(cancelled) = self.jobs.lock().expect("active jobs lock").get(id) {
            cancelled.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn remove(&self, id: &str) {
        self.jobs.lock().expect("active jobs lock").remove(id);
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionProgressEvent {
    pub job_id: String,
    pub percent: f32,
}
