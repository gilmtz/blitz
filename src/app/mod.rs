use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use egui::Key;
use log::{log, Level};
use models::{ImageInfo, Rating};
use ron::ser::PrettyConfig;

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

    fn update_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                self.setup_menu_bar(ctx, ui);
            });
        });
    }

    fn handle_user_input(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Open folder…").clicked() {
            
                // save_culling_progress(&self.photo_dir, photos);
    
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = pick_folder() {
                self.open_folder_action(ui, path);
            }
            #[cfg(target_arch = "wasm32")]
            let _ = self.open_folder_action();
        
            
        }

        if ui.button("Commit choices").clicked() {
            self.commit_choices(ui);
        }

        // #[cfg(not(target_arch = "wasm32"))]
        // if ui.button("Load next textures").clicked() {
        //     let mut photos = (&self.photos).to_owned();
        //     let max_texture_count = (&self.max_texture_count).to_owned();
        //     let thread_ctx = ui.ctx().clone();

        //     let _handler = thread::spawn(move || {
        //         open_folder_native::load_all_textures_into_memory(&mut photos, thread_ctx, max_texture_count);
        //     });
        // }

        if ctx.input(|i| i.key_pressed(Key::D)) {
            log!(Level::Info, "D pressed");
            go_to_next_picture(self);
        }

        if ctx.input(|i| i.key_pressed(Key::A)) {
            go_to_previous_picture(self)
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            let photos_index = self.photos_index;
            self.photos.write().unwrap()[photos_index].rating = Rating::Remove;
            self.photos.write().unwrap()[photos_index].texture = Arc::new(Mutex::new(None));
            go_to_next_picture(self);
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            let photos_index = self.photos_index;
            self.photos.write().unwrap()[photos_index].rating = Rating::Approve;
            go_to_next_picture(self);
        }
    }

    fn commit_choices(&mut self, ui: &mut egui::Ui) {
        match fs::create_dir_all(&self.get_chaffe_dir(&(self.photo_dir.clone())).clone()) {
            Ok(it) => it,
            Err(_err) => todo!("handle when we can't make directories later"),
        };
    
        match fs::create_dir_all(&self.get_wheat_dir(&(self.photo_dir.clone())).clone()) {
            Ok(it) => it,
            Err(_err) => todo!("handle when we can't make directories later"),
        };
        let chaffe_dir = &self.get_chaffe_dir(&(self.photo_dir.clone()));
        let wheat_dir = &self.get_wheat_dir(&(self.photo_dir.clone()));
        if let Ok(photos) = self.photos.try_read() {
            commit_culling(&*photos,chaffe_dir,wheat_dir);
        }   
        
        #[cfg(not(target_arch = "wasm32"))]
        self.open_folder_action(ui, self.photo_dir.clone());
    }
    
    fn get_chaffe_dir(&mut self, root_dir: &PathBuf) -> PathBuf {
        match &self.chaffe_dir_target {
            Some(target_dir) => target_dir.clone(),
            None => {
                let mut chaffe_dir = root_dir.clone();
                chaffe_dir.push("chaffe");
                return chaffe_dir;
            },
        }
    }
    
    fn get_wheat_dir(&mut self, root_dir: &PathBuf) -> PathBuf {
        match &self.wheat_dir_target {
            Some(target_dir) => target_dir.clone(),
            None => {
                let mut wheat_dir = root_dir.clone();
                wheat_dir.push("wheat");
                return wheat_dir;
            },
        }
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
            save_culling_progress(&self.photo_dir, &*photos);
        }

    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.update_top_panel(ctx);

        self.update_left_panel(ctx);

        // self.update_right_panel(ctx);

        self.update_center_panel(ctx);
    }
}

fn go_to_next_picture(template_app: &mut BlitzApp) {
    log::info!("Go to next picture");
    if let Ok(photos) = template_app.photos.try_read() {
        match get_next_picture_index(template_app.photos_index.clone(), &*photos) {
            Some(index) => {
                log::info!("Moving to index: {}", index);
                template_app.photos_index = index;
    
            },
            None => todo!("we rated everything so now we die"),
        }
    }
}

fn go_to_previous_picture(template_app: &mut BlitzApp) {
    if let Ok(photos) = template_app.photos.try_read() {
        match get_previous_picture_index(template_app.photos_index.clone(), &*photos) {
            Some(index) => template_app.photos_index = index,
            None => todo!("we rated everything so now we die"),
        }
    }
}

fn commit_culling(
    photos: &Vec<ImageInfo>,
    chaffe_dir: &PathBuf,
    wheat_dir: &PathBuf,
) -> Vec<Result<(), io::Error>> {
    let mut committing_results = vec![];
    for image in photos.iter() {
        handle_image_cull(chaffe_dir, wheat_dir, &mut committing_results, image);
    }
    return committing_results;
}

fn handle_image_cull(chaffe_dir: &PathBuf, wheat_dir: &PathBuf, committing_results: &mut Vec<Result<(), io::Error>>, image: &ImageInfo) {
    match image.rating {
        Rating::Unrated => {}
        Rating::Approve => {
            committing_results.push(move_image_into_dir(&wheat_dir, &image));
        }
        Rating::Remove => {
            committing_results.push(move_image_into_dir(&chaffe_dir, &image));
        }
    }
}

fn move_image_into_dir(destination_dir: &PathBuf, image: &ImageInfo) -> Result<(), std::io::Error> {
    let mut processed_image_destination = destination_dir.clone();
    processed_image_destination.push(image.image_name.clone());
    fs::rename(image.path_processed.clone(), processed_image_destination)?;

    match &image.path_raw {
        Some(path_raw) => {
            let mut raw_image_destination = destination_dir.clone();
            raw_image_destination.push(image.image_name.clone());
            raw_image_destination.set_extension("RAF");
            fs::rename(path_raw.clone(), raw_image_destination)?;
        }
        None => {}
    }
    Ok(())
}

fn save_culling_progress(photo_dir: &PathBuf, photos: &Vec<ImageInfo>) -> io::Result<()>  {
    // This handles the initial opening case
    if photos.is_empty() {
        return Ok(());
    }
    let mut blitz_dir = photo_dir.clone();
    blitz_dir.push(".blitz");

    match fs::create_dir_all(blitz_dir.clone()) {
        Ok(_dir) => {}
        Err(_err) => {}
    };

    blitz_dir.push("storage.ron");

    // Serialize and write
    let ron_str = ron::ser::to_string_pretty(&photos, PrettyConfig::new())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(blitz_dir, ron_str)?;

    Ok(())
}

fn get_next_picture_index(
    starting_index: usize,
    photos: &Vec<ImageInfo>,
) -> Option<usize> {
    log::info!("Get next picture index");
    let mut candidate_index = starting_index.clone();
    loop {
        candidate_index += 1;
        if candidate_index >= photos.len() {
            candidate_index = 0
        }
        if photos[candidate_index].rating == Rating::Unrated {
            return Some(candidate_index);
        }
        if starting_index == candidate_index {
            return None;
        }
    }
}

fn get_previous_picture_index(
    starting_index: usize,
    photos: &Vec<ImageInfo>,
) -> Option<usize> {
    let mut candidate_index = starting_index.clone();
    loop {
        if candidate_index == 0 {
            candidate_index = photos.len() - 1;
        } else {
            candidate_index = candidate_index - 1;
        }
        if photos[candidate_index].rating == Rating::Unrated {
            return Some(candidate_index);
        }
        if starting_index == candidate_index {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_picture_index_no_ratings() {
        let mut test_photos = Vec::new();
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into()
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),

        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),

        });

        let next_picture_index = get_next_picture_index(0, &test_photos);
        assert_eq!(Some(1), next_picture_index);
        let next_picture_index = get_next_picture_index(1, &test_photos);
        assert_eq!(Some(2), next_picture_index);
        let next_picture_index = get_next_picture_index(2, &test_photos);
        assert_eq!(Some(0), next_picture_index);

        let next_picture_index = get_previous_picture_index(0, &test_photos);
        assert_eq!(Some(2), next_picture_index);
        let next_picture_index = get_previous_picture_index(1, &test_photos);
        assert_eq!(Some(0), next_picture_index);
        let next_picture_index = get_previous_picture_index(2, &test_photos);
        assert_eq!(Some(1), next_picture_index);
    }

    #[test]
    fn test_get_next_picture_index_full_list() {
        let mut test_photos = Vec::new();
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });

        let next_picture_index = get_next_picture_index(0, &test_photos);
        assert_eq!(None, next_picture_index);
        let next_picture_index = get_next_picture_index(1, &test_photos);
        assert_eq!(None, next_picture_index);
        let next_picture_index = get_next_picture_index(2, &test_photos);
        assert_eq!(None, next_picture_index);

        let previous_picture_index = get_previous_picture_index(0, &test_photos);
        assert_eq!(None, previous_picture_index);
        let previous_picture_index = get_previous_picture_index(1, &test_photos);
        assert_eq!(None, previous_picture_index);
        let previous_picture_index = get_previous_picture_index(2, &test_photos);
        assert_eq!(None, previous_picture_index);
    }

    #[test]
    fn test_get_next_picture_index_skip_rated() {
        let mut test_photos = Vec::new();
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
            data: [].into(),
        });

        let next_picture_index = get_next_picture_index(0, &test_photos);
        assert_eq!(Some(2), next_picture_index);
        let next_picture_index = get_next_picture_index(1, &test_photos);
        assert_eq!(Some(2), next_picture_index);
        let next_picture_index = get_next_picture_index(2, &test_photos);
        assert_eq!(Some(2), next_picture_index);
        let next_picture_index = get_next_picture_index(3, &test_photos);
        assert_eq!(Some(2), next_picture_index);

        let previous_picture_index = get_previous_picture_index(0, &test_photos);
        assert_eq!(Some(2), previous_picture_index);
        let previous_picture_index = get_previous_picture_index(1, &test_photos);
        assert_eq!(Some(2), previous_picture_index);
        let previous_picture_index = get_previous_picture_index(2, &test_photos);
        assert_eq!(Some(2), previous_picture_index);
        let previous_picture_index = get_previous_picture_index(3, &test_photos);
        assert_eq!(Some(2), previous_picture_index);
    }

    #[test]
    fn test_commit_culling() {
        let temp_path = PathBuf::from("tmp");
        fs::create_dir_all(&temp_path).unwrap();

        copy_test_images_to_dir();

        let mut test_photos = Vec::new();

        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("tmp/1.jpg"),
            path_raw: None,
            rating: Rating::Remove,
            texture: Arc::new(Mutex::new(None)),
            image_name: "1.jpg".to_string(),
            data: [].into(), // Added field
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("tmp/2.jpg"),
            path_raw: None,
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "2.jpg".to_string(),
            data: [].into(), // Added field
        });
        test_photos.push(ImageInfo {
            path_processed: PathBuf::from("tmp/3.jpg"),
            path_raw: None,
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "3.jpg".to_string(),
            data: [].into(), // Added field
        });

        let temp_path = PathBuf::from("tmp");
        let chaffe_path = PathBuf::from("tmp/chaffe");
        let wheat_path = PathBuf::from("tmp/wheat");

        fs::create_dir_all(&chaffe_path).unwrap();
        fs::create_dir_all(&wheat_path).unwrap();

        commit_culling(&test_photos, &chaffe_path, &wheat_path);

        // Confirm first image was moved to chaffe folder and no longer exists in original folder
        assert_identical_files("assets/samples/1.jpg", "tmp/chaffe/1.jpg");
        assert!(!PathBuf::from("tmp/1.jpg").exists());

        // Confirm the second unrated image was not moved
        assert!(PathBuf::from("tmp/2.jpg").exists());
        assert!(!PathBuf::from("tmp/wheat/2.jpg").exists());
        assert!(!PathBuf::from("tmp/chaffe/2.jpg").exists());

        // Confirm the third image was moved to wheat folder and no longer exists in original folder
        assert_identical_files("assets/samples/3.jpg", "tmp/wheat/3.jpg");
        assert!(!PathBuf::from("tmp/3.jpg").exists());

        fs::remove_dir_all(&chaffe_path).unwrap();
        fs::remove_dir_all(&wheat_path).unwrap();
        fs::remove_dir_all(&temp_path).unwrap();
    }

    fn copy_test_images_to_dir() {
        fs::copy(
            PathBuf::from("assets/samples/1.jpg"),
            PathBuf::from("tmp/1.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/2.jpg"),
            PathBuf::from("tmp/2.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/3.jpg"),
            PathBuf::from("tmp/3.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/4.jpg"),
            PathBuf::from("tmp/4.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/5.jpg"),
            PathBuf::from("tmp/5.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/6.jpg"),
            PathBuf::from("tmp/6.jpg"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from("assets/samples/7.jpg"),
            PathBuf::from("tmp/7.jpg"),
        )
        .unwrap();
    }

    fn assert_identical_files(src_path_string: &str, dest_path_string: &str) {
        let source_bytes = fs::read(&src_path_string).unwrap();
        let dest_bytes = fs::read(&dest_path_string).unwrap();
        assert_eq!(source_bytes, dest_bytes);
    }
}

mod models;
// mod right_panel;
mod left_panel;
mod center_panel;
mod context_menu;
mod menu_bar;
#[cfg(not(target_arch = "wasm32"))]
mod open_folder_native;
#[cfg(target_arch = "wasm32")]
mod open_folder_wasm;