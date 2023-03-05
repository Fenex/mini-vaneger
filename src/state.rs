use std::{fs::File, path::Path};

use druid::{
    im, {Data, Lens},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Data, Default, Serialize, Deserialize, Lens)]
pub struct Config {
    pub addons: im::Vector<Item>,
}

#[derive(Debug, Clone, Data, Default, Serialize, PartialEq, Deserialize, Lens)]
pub struct Item {
    pub name: String,
    pub enabled: bool,
    #[serde(flatten)]
    #[data(ignore)]
    extra: im::HashMap<String, Value>,
}

#[derive(Debug, Clone, Data, Serialize, Deserialize, Lens)]
pub struct AppState {
    pub path: String,
    pub config: Config,
}

impl AppState {
    pub fn new(path: &Path) -> Self {
        let f = File::open(path).unwrap();
        let config = serde_json::from_reader(f).unwrap();

        Self {
            path: path.to_string_lossy().to_string(),
            config,
        }
    }
}
