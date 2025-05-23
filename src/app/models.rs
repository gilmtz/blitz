use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::BlitzApp;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ImageInfo {
    pub path_processed: PathBuf,
    pub path_raw: Option<PathBuf>,
    #[serde(skip)]
    pub data: Arc<[u8]>,
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
            photos: Arc::new(Vec::new().into()),
            photo_dir: PathBuf::new(),
            max_texture_count: 200,
            uv_size: 1.0,
            wheat_dir_target: None,
            chaffe_dir_target: None,
        }
    }
}
