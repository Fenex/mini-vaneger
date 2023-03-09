use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
};

use druid::{
    im, {Data, Lens},
};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Data, Default, Serialize, Deserialize, Lens)]
pub struct AddonsCfg {
    pub addons: im::Vector<Item>,
    #[serde(skip)]
    pub path: String,
}

impl AddonsCfg {
    pub const ENV_ADDONS_DIR_PATH: &str = "MINIVANEGER_ADDONS_DIR_PATH";
    pub const ADDONS_FILE_PATH: &str = "./scripts/ls.json";

    pub fn file_name() -> &'static str {
        Path::new(Self::ADDONS_FILE_PATH)
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap()
    }

    fn path_to_json(&self) -> PathBuf {
        self.scripts_directory().join(Self::file_name())
    }

    /// returns full path to ./scripts/ls.json
    pub fn scripts_directory(&self) -> PathBuf {
        dunce::canonicalize(&self.path).unwrap()
    }

    pub fn load() -> Self {
        let path_dir = if let Ok(path) = std::env::var(Self::ENV_ADDONS_DIR_PATH) {
            Cow::Owned(PathBuf::from(path))
        } else {
            Cow::Borrowed(Path::new(Self::ADDONS_FILE_PATH).parent().unwrap())
        };

        let json_file = path_dir.join(Self::file_name());

        if !json_file.is_file() {
            return Default::default();
        }

        let mut config = File::open(&json_file)
            .map(serde_json::from_reader::<_, Self>)
            .map_or_else(
                |e| {
                    warn!("File open {:?}: {:?}", &json_file, e);
                    Ok(Default::default())
                },
                |e| e,
            )
            .map_or_else(
                |e| {
                    warn!("Deserialize {:?}: {:?}", &json_file, e);
                    Default::default()
                },
                |e| e,
            );

        config.path = path_dir.into_owned().to_string_lossy().to_string();
        config
    }

    pub fn save(&self) {
        let path = self.path_to_json();

        let result = File::options()
            .write(true)
            .truncate(true)
            .open(&path)
            .map(|w| serde_json::to_writer_pretty(w, self));

        if let Ok(Ok(())) = result {
            log::info!("`{:?}`, saved JSON file.", &path);
        } else {
            log::warn!("`{:?}`, failed to save JSON file", &path);
        }
    }
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
    pub config: AddonsCfg,
    pub settings: SettingsCfg,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: AddonsCfg::load(),
            settings: SettingsCfg::load(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Data, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    #[default]
    En,
    Ru,
}

#[derive(Debug, Default, Clone, Data, Serialize, Deserialize, PartialEq, Lens)]
pub struct SettingsCfg {
    pub resource_dir: String,
    #[serde(skip)]
    pub resource_dir_validated: bool,
    pub language: Language,
    #[data(ignore)]
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl SettingsCfg {
    pub const ENV_FILE_PATH: &str = "MINIVANEGER_SETTINGS_FILE_PATH";
    pub const SETTINGS_FILE_PATH: &str = "minivaneger.ron";

    pub fn load() -> Self {
        let path = std::env::var(Self::ENV_FILE_PATH)
            .map(PathBuf::from)
            .ok()
            .and_then(|p| {
                if p.is_file() {
                    Some(Cow::Owned(p))
                } else {
                    None
                }
            })
            .unwrap_or(Cow::Borrowed(Path::new(Self::SETTINGS_FILE_PATH)));

        let mut this = match File::open(&path) {
            Ok(f) => {
                let rdr = BufReader::new(f);
                match ron::de::from_reader::<_, Self>(rdr) {
                    Ok(mut this) => {
                        this.path = Some(path.to_path_buf());
                        this
                    }
                    Err(_) => Default::default(),
                }
            }
            Err(_) => Default::default(),
        };

        if !this.resource_dir.is_empty() {
            let path_dir = Path::new(&this.resource_dir);
            if path_dir.join("tabutask.prm").is_file() {
                this.resource_dir_validated = true;
            }
        }

        this
    }

    pub fn file(&self) -> Cow<Path> {
        self.path
            .as_ref()
            .map(|p| Cow::Borrowed(p.as_path()))
            .unwrap_or_else(|| Cow::Owned(PathBuf::from(Self::SETTINGS_FILE_PATH)))
    }

    pub fn save(&self) {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.file())
            .map(|f| ron::ser::to_writer_pretty(f, self, Default::default()))
            .map_err(|e| warn!("Error SettingsCfg::save: {:?}", e))
            .ok();
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct ResourceDirectoryState {
    pub resource_dir: String,
    pub resource_dir_validated: bool,
}

pub struct Settings2ResourceDirectoryState;
impl Lens<SettingsCfg, ResourceDirectoryState> for Settings2ResourceDirectoryState {
    fn with<V, F: FnOnce(&ResourceDirectoryState) -> V>(&self, data: &SettingsCfg, f: F) -> V {
        f(&ResourceDirectoryState {
            resource_dir: data.resource_dir.clone(),
            resource_dir_validated: data.resource_dir_validated,
        })
    }

    fn with_mut<V, F: FnOnce(&mut ResourceDirectoryState) -> V>(
        &self,
        data: &mut SettingsCfg,
        f: F,
    ) -> V {
        f(&mut ResourceDirectoryState {
            resource_dir: data.resource_dir.clone(),
            resource_dir_validated: data.resource_dir_validated,
        })
    }
}
