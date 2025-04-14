use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use egui::Key;
use open_folder_wasm::ImageFile;
use ron::ser::PrettyConfig;

use super::{open_folder_wasm, BlitzApp};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ImageInfo {
    pub path_processed: PathBuf,
    pub path_raw: Option<PathBuf>,
    pub file_bytes: Vec<u8>,
    pub rating: Rating,
    #[serde(skip)]
    pub texture: Arc<Mutex<Option<egui::TextureHandle>>>,
    pub image_name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
pub enum Rating {
    Unrated,
    Approve,
    Remove,
}

impl Default for BlitzApp {
    fn default() -> Self {
        Self {
            photos_index: 0,
            photos: Vec::new().into(),
            image_files: Vec::new().into(),
            photo_dir: PathBuf::new(),
            max_texture_count: 200,
            uv_size: 1.0,
            wheat_dir_target: None,
            chaffe_dir_target: None,
        }
    }
}