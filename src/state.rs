use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use druid::{im, Data, Lens};

use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const ENV_ADDONS_DIR_PATH: &str = "MINIVANEGER_ADDONS_DIR_PATH";

fn load_addon(path: &Path) -> Option<AddonsCfg> {
    if !path.join("vss.exe").is_file() {
        trace!("{:?}: `vss.exe` not found or not a file, skipped", path);
        None?
    }

    if !path.join("scripts").is_dir() {
        trace!(
            "{:?}: `scripts` not found or not a directory, skipped",
            path
        );
        None?
    }

    let json_file = dunce::canonicalize(path.join("ls.json")).unwrap();

    File::open(&json_file)
        .map_or_else(
            |e| {
                warn!("Error: file open {:?}: {:?}", &json_file, e);
                None
            },
            Some,
        )
        .map(serde_json::from_reader::<_, AddonsCfg>)?
        .map_or_else(
            |e| {
                warn!("Error: deserialize {:?}: {:?}", &json_file, e);
                None
            },
            Some,
        )
        .map(|mut cfg| {
            cfg.dirname = json_file
                .parent()
                .unwrap()
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .to_string()
                .into();
            cfg.config_path = json_file.to_string_lossy().to_string().into();
            cfg
        })
}

fn load_addons() -> Vec<AddonsCfg> {
    trace!("load_addons");
    let path_dir = if let Ok(path) = std::env::var(ENV_ADDONS_DIR_PATH) {
        PathBuf::from(path)
    } else {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned()
    };

    trace!("directory to addons: `{:?}`", path_dir);

    let mut addons = vec![];

    if let Ok(mut read_dir) = std::fs::read_dir(path_dir) {
        while let Some(Ok(entry)) = read_dir.next() {
            if let Some(cfg) = load_addon(&entry.path()) {
                addons.push(cfg);
            }
        }
    }

    addons
}

#[derive(Debug, Clone, Data, Default, Serialize, Deserialize, Lens)]
pub struct AddonsCfg {
    pub addons: im::Vector<Item>,
    #[serde(skip)]
    pub config_path: Arc<String>,
    #[serde(skip)]
    pub dirname: Arc<String>,
    #[serde(flatten)]
    #[data(ignore)]
    extra: im::HashMap<String, Value>,
}

impl AddonsCfg {
    pub fn save(&self) {
        let path = Path::new(&*self.config_path);

        let result = File::options()
            .write(true)
            .truncate(true)
            .open(path)
            .map(|w| serde_json::to_writer_pretty(w, self));

        if let Ok(Ok(())) = result {
            log::info!("`{:?}`, saved JSON file.", path);
        } else {
            log::warn!("`{:?}`, failed to save JSON file", path);
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
    pub vss: im::Vector<AddonsCfg>,
    pub settings: SettingsCfg,
}

impl AppState {
    pub fn new() -> Self {
        trace!("AppState::new");
        let t_vss = std::thread::spawn(load_addons);
        let t_settings = std::thread::spawn(SettingsCfg::load);

        Self {
            vss: t_vss.join().unwrap().into(),
            settings: t_settings.join().unwrap(),
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
