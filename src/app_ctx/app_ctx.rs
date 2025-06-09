use std::sync::Arc;

use rust_extensions::AppStates;

pub struct AppContext {
    pub states: Arc<AppStates>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            states: Arc::new(AppStates::create_initialized()),
        }
    }
}
