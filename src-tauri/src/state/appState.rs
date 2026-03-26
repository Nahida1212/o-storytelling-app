
use std::sync::Mutex;
use crate::{
    config::{
        self,
        appConfig::{self, load_config, AppConfig},
    },
    state::appState,
};

#[derive(Debug)]
pub struct AppState {
    config: Mutex<AppConfig>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
        }
    }
}

pub fn load_state() -> Result<String, String> {
    // read from disk

    
    Ok(" is ok".to_owned())
}
