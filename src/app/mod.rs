use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use egui::Key;
use file_operations::save_culling_progress;
use log::{log, Level};
use models::{ImageInfo, Rating};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct BlitzApp {
    #[serde(skip)] // This how you opt-out of serialization of a field
    pub photos_index: usize,
    #[serde(skip)]
    pub uv_size: f32,
    #[serde(skip)]
    pub photos: Arc<RwLock<Vec<ImageInfo>>>,
    pub photo_dir: PathBuf,
    pub wheat_dir_target: Option<PathBuf>,
    pub chaffe_dir_target: Option<PathBuf>,
    pub max_texture_count: usize,
}

impl BlitzApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        if let Some(storage) = cc.storage {
            let persisted_state: BlitzApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            return persisted_state;
        }

        Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

impl eframe::App for BlitzApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);

        if let Ok(photos) = self.photos.try_read() {
            let _ = save_culling_progress(&self.photo_dir, &photos);
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.update_top_panel(ctx);

        self.update_left_panel(ctx);

        self.update_right_panel(ctx);

        self.update_center_panel(ctx);
    }
}

mod context_menu;
mod file_operations;
mod models;
mod navigation;
#[cfg(not(target_arch = "wasm32"))]
mod open_folder_native;
#[cfg(target_arch = "wasm32")]
mod open_folder_wasm;
mod panels;
