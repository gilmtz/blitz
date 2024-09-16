use std::{
    fs::{self},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
};

use egui::{ColorImage, Key, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)] // This how you opt-out of serialization of a field
    photos_index: usize,

    #[serde(skip)]
    photos: Vec<Arc<RwLock<ImageInfo>>>,

    photo_dir: PathBuf,
    show_deferred_viewport: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct ImageInfo {
    path_processed: PathBuf,
    path_raw: Option<PathBuf>,
    rating: Rating,
    texture: Arc<Mutex<Option<egui::TextureHandle>>>,
    image_name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Rating {
    Skip,
    Approve,
    Remove,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            photos_index: 0,
            photos: Vec::new().into(),
            show_deferred_viewport: AtomicBool::new(false).into(),
            photo_dir: PathBuf::from("C:\\Users\\gilbe\\Pictures\\Photo_Dump"),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Queue");

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.photos.len() > 0 {
                    for photo in self.photos.iter() {
                        match photo.read().unwrap().rating {
                            Rating::Skip => {
                                let texture_mutex = photo.read().unwrap().texture.clone();
                                match texture_mutex.try_lock() {
                                    Ok(texture_handle) => {
                                        match *texture_handle {
                                            Some(ref texture) => {
                                                ui.image((texture.id(), Vec2::new(100.0, 100.0)))
                                            }
                                            None => ui.label("texture not ready"),
                                        };
                                    }
                                    Err(_) => {}
                                };
                            }
                            Rating::Approve => {}
                            Rating::Remove => {}
                        }
                    }
                }
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Keep");

            if self.photos.len() > 0 {
                for photo in self.photos.iter().rev() {
                    match photo.read().unwrap().rating {
                        Rating::Skip => {}
                        Rating::Approve => {
                            let texture_mutex = photo.read().unwrap().texture.clone();
                            match texture_mutex.try_lock() {
                                Ok(texture_handle) => {
                                    match *texture_handle {
                                        Some(ref texture) => {
                                            ui.image((texture.id(), Vec2::new(100.0, 100.0)))
                                        }
                                        None => ui.label("texture not ready"),
                                    };
                                }
                                Err(_) => {}
                            };
                        }
                        Rating::Remove => {}
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            if ui.button("Open folder…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.photo_dir = path;
                    init_photos_state(self.photo_dir.clone(), &mut self.photos);

                    let mut photos = (&self.photos).to_owned();
                    let thread_ctx = ui.ctx().clone();

                    let _handler = thread::spawn(move || {
                        load_images_into_memory(&mut photos, &thread_ctx);
                    });
                }
            }

            if ui.button("Commit choices").clicked() {
                commit_culling(&self.photos, self.photo_dir.clone());
            }

            if ctx.input(|i| i.key_pressed(Key::D)) {
                go_to_next_picture(self);
            }

            if ctx.input(|i| i.key_pressed(Key::A)) {
                go_to_previous_picture(self)
            }

            if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                self.photos[self.photos_index].write().unwrap().rating = Rating::Remove;
                go_to_next_picture(self);
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                self.photos[self.photos_index].write().unwrap().rating = Rating::Approve;
                go_to_next_picture(self);
            }

            if self.photos.len() > 0 {
                let texture_mutex = self.photos[self.photos_index]
                    .read()
                    .unwrap()
                    .texture
                    .clone();
                match texture_mutex.try_lock() {
                    Ok(texture_handle) => {
                        match *texture_handle {
                            Some(ref texture) => {
                                ui.add(egui::Image::new(texture).max_width(1000.0))
                                // ui.image((texture.id(), Vec2::new(1000.0, 1000.0)))
                            }
                            None => ui.label("texture not ready"),
                        };
                    }
                    Err(_) => {}
                };
            }

            if self.show_deferred_viewport.load(Ordering::Relaxed) {
                let show_deferred_viewport = self.show_deferred_viewport.clone();
                ctx.show_viewport_deferred(
                    egui::ViewportId::from_hash_of("deferred_viewport"),
                    egui::ViewportBuilder::default()
                        .with_title("Deferred Viewport")
                        .with_inner_size([200.0, 100.0]),
                    move |ctx, class| {
                        assert!(
                            class == egui::ViewportClass::Deferred,
                            "This egui backend doesn't support multiple viewports"
                        );

                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label("Hello from deferred viewport");
                        });
                        if ctx.input(|i| i.viewport().close_requested()) {
                            // Tell parent to close us.
                            show_deferred_viewport.store(false, Ordering::Relaxed);
                        }
                    },
                );
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn init_photos_state(photo_dir: PathBuf, photos: &mut Vec<Arc<RwLock<ImageInfo>>>) {
    let paths = fs::read_dir(photo_dir.clone()).unwrap();
    for path in paths.take(50) {
        match path {
            Ok(ref x) => {
                let file_extension = match x.path().extension() {
                    Some(extension) => extension.to_owned(),
                    None => todo!("need to handle when we can't get the file extension"),
                };
                if file_extension == "JPG" {
                    let filename = x.path().file_name().unwrap().to_str().unwrap().to_string();

                    let image_info = ImageInfo {
                        path_processed: x.path().clone(),
                        path_raw: get_raw_variant(x.path().clone()),
                        rating: Rating::Skip,
                        texture: Arc::new(Mutex::new(None)),
                        image_name: filename,
                    };
                    photos.push(Arc::new(RwLock::new(image_info)));
                }
            }
            Err(_) => todo!("need to handle when the path errors out"),
        }
    }
}

fn get_raw_variant(mut processed_path: PathBuf) -> Option<PathBuf> {
    if processed_path.pop() {
        processed_path.push(".RAF");
        return Some(processed_path);
    } else {
        return None;
    }
}

fn load_images_into_memory(photos: &mut Vec<Arc<RwLock<ImageInfo>>>, ctx: &egui::Context) {
    for image_info in photos {
        let data = match fs::read(&image_info.read().unwrap().path_processed) {
            Ok(result) => result,
            Err(_) => todo!("handle when we can't read the file"),
        };
        let image_data = load_image_from_memory(&data);

        let texture_handle = match image_data {
            Ok(color_image) => {
                let texture_id = image_info
                    .read()
                    .unwrap()
                    .path_processed
                    .clone()
                    .as_mut_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();
                Some(ctx.load_texture(texture_id, color_image, Default::default()))
            }
            Err(_) => None,
        };

        image_info.write().unwrap().texture = Arc::new(Mutex::new(texture_handle));
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn go_to_next_picture(template_app: &mut TemplateApp) {
    let starting_index = template_app.photos_index;
    loop {
        template_app.photos_index += 1;
        if template_app.photos_index >= template_app.photos.len() {
            template_app.photos_index = 0
        }
        if template_app.photos[template_app.photos_index]
            .read()
            .unwrap()
            .rating
            == Rating::Skip
        {
            break;
        }
        if starting_index == template_app.photos_index {
            commit_culling(&template_app.photos, template_app.photo_dir.clone());
            todo!("We finished culling our pictures move to confirmation screen")
        }
    }
}

fn go_to_previous_picture(template_app: &mut TemplateApp) {
    if template_app.photos_index == 0 {
        template_app.photos_index = template_app.photos.len() - 1
    }
    template_app.photos_index -= 1;
}

fn commit_culling(photos: &Vec<Arc<RwLock<ImageInfo>>>, root_dir: PathBuf) {
    let mut chaffe_dir = root_dir.clone();
    chaffe_dir.push("chaffe");
    let mut wheat_dir = root_dir.clone();
    wheat_dir.push("wheat");

    match fs::create_dir(chaffe_dir.clone()) {
        Ok(it) => it,
        Err(_err) => todo!("handle when we can't make directories later"),
    };

    match fs::create_dir(wheat_dir.clone()) {
        Ok(it) => it,
        Err(_err) => todo!("handle when we can't make directories later"),
    };

    for image in photos.iter() {
        match image.read().unwrap().rating {
            Rating::Skip => {}
            Rating::Approve => {
                copy_image_into_dir(&wheat_dir, &image.read().unwrap())
            }
            Rating::Remove => {
                copy_image_into_dir(&chaffe_dir, &image.read().unwrap())
            }
        }
    }
}

fn copy_image_into_dir(destination_dir: &PathBuf, image: &ImageInfo) {
    let mut proccessed_image_destination = destination_dir.clone();
    proccessed_image_destination.push(image.image_name.clone());
    let _ = fs::copy(image.path_processed.clone(), proccessed_image_destination);

    match &image.path_raw {
        Some(path_raw) => {
            let mut raw_image_destination = destination_dir.clone();
            raw_image_destination.push(image.image_name.clone());
            let _ = fs::copy(path_raw.clone(), raw_image_destination);
        },
        None => {},
    }
    
}