use std::{
    ffi::OsString,
    fs::{self},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use egui::{ColorImage, TextureHandle};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{console, js_sys::{self, ArrayBuffer, AsyncIterator, Promise, Uint8Array}, window, DirectoryPickerOptions, File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemHandle, FileSystemHandleKind};

use super::{BlitzApp, ImageInfo, Rating};

impl BlitzApp {
    // open folder handles initialization of the app and the loading of the images
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_folder_action(&mut self, ui: &mut egui::Ui, path: PathBuf) {
        self.photo_dir = path.clone();

        // Restore state from .blitz folder
        let mut blitz_dir = self.photo_dir.clone();
        blitz_dir.push(".blitz");
        blitz_dir.push("storage.ron");

        match fs::read(blitz_dir.clone()) {
            Ok(seralized_ron) => {
                self.photos = Vec::new().into();
                match &ron::de::from_bytes::<Vec<Arc<RwLock<ImageInfo>>>>(&seralized_ron) {
                    Ok(stored_state) => {
                        let stored_images = stored_state.clone();
                        init_photos_state(
                            self.photo_dir.clone(),
                            &mut self.photos,
                            Some(stored_images),
                        );
                    }
                    Err(_) => todo!("failed to deserialized the previous state"),
                }
            }
            Err(_) => {
                init_photos_state(self.photo_dir.clone(), &mut self.photos, None);
            }
        }

        self.photos_index = get_first_unrated_image_index(&self.photos);

        let mut photos = (&self.photos).to_owned();
        let max_texture_count = (&self.max_texture_count).to_owned();
        let thread_ctx = ui.ctx().clone();

        let _handler = thread::spawn(move || {
            load_all_textures_into_memory(&mut photos, thread_ctx, max_texture_count);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn init_photos_state(
    photo_dir: PathBuf,
    photos: &mut Vec<Arc<RwLock<ImageInfo>>>,
    stored_photos: Option<Vec<Arc<RwLock<ImageInfo>>>>,
) {
    let paths = fs::read_dir(photo_dir.clone()).unwrap();
    for path in paths {
        let x = path.unwrap();
        match x.path().is_file() {
            false => {} // TODO: handle folders recursively?
            true => {
                if let Some(image_info) = init_image_info(x, &stored_photos) {
                    photos.push(image_info);
                }
            }
        }
    }
}

fn get_first_unrated_image_index(photos: &Vec<Arc<RwLock<ImageInfo>>>) -> usize {
    let mut counter: usize = 0;
    for image_lock in photos {
        let image = image_lock.read().unwrap().clone();
        if image.rating == Rating::Unrated {
            return counter;
        }
        counter += 1;
    }
    return counter;
}

pub fn load_all_textures_into_memory(
    photos: &mut Vec<Arc<RwLock<ImageInfo>>>,
    ctx: egui::Context,
    max_texture_count: usize,
) {
    let mut texture_counter = 0;
    for image_info in photos {
        if image_info.read().unwrap().rating == Rating::Unrated
            && texture_counter < max_texture_count
        {
            if let Some(_) = load_texture_into_memory(image_info, ctx.clone()) {
                texture_counter += 1
            }
        }
    }
}

fn init_image_info(
    dir_entry: fs::DirEntry,
    stored_photos: &Option<Vec<Arc<RwLock<ImageInfo>>>>,
) -> Option<Arc<RwLock<ImageInfo>>> {
    let entry_path = dir_entry.path();
    let file_extension = match entry_path.extension() {
        Some(extension) => extension.to_owned(),
        None => {
            println!("Couldn't get extension for {:?}", entry_path);
            return None;
        },
    };
    if !is_file_extension_supported(file_extension) {
        return None;
    }
    let filename = dir_entry.path().file_name().unwrap().to_str().unwrap().to_string();

    let image_info = ImageInfo {
        path_processed: dir_entry.path().clone(),
        path_raw: get_raw_variant(&dir_entry.path()),
        rating: get_rating_for_image(stored_photos, dir_entry.path().clone()),
        texture: Arc::new(Mutex::new(None)),
        image_name: filename,
        data: vec![]
    };
    Some(Arc::new(RwLock::new(image_info)))
}

fn load_texture_into_memory(
    image_info: &mut Arc<RwLock<ImageInfo>>,
    ctx: egui::Context,
) -> Option<TextureHandle> {
    let data = match fs::read(&image_info.read().unwrap().path_processed) {
        Ok(result) => result,
        Err(_) => return None, // If we can't read the image we just skip it
    };
    let image_data = create_image(&data);

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
        Err(_) => return None,
    };

    image_info.write().unwrap().texture = Arc::new(Mutex::new(texture_handle.clone()));
    texture_handle
}

fn is_file_extension_supported(extension: OsString) -> bool {
    if extension == "JPG" {
        return true;
    }
    if extension == "jpg" {
        return true;
    }
    return false;
}

fn get_raw_variant(processed_path: &PathBuf) -> Option<PathBuf> {
    let mut raw_path = processed_path.clone();
    match raw_path.set_extension("RAF") {
        true => Some(raw_path),
        false => None,
    }
}

fn get_rating_for_image(
    stored_photos: &Option<Vec<Arc<RwLock<ImageInfo>>>>,
    image_path: PathBuf,
) -> Rating {
    match stored_photos {
        Some(photos) => {
            for image_lock in photos {
                let image = image_lock.read().unwrap().clone();
                if image.path_processed == image_path {
                    return image.rating;
                }
            }
            return Rating::Unrated;
        }
        None => Rating::Unrated,
    }
}

fn create_image(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_raw_variant() {
        let path = PathBuf::from("/tmp/DSC55555.jpg");
        let raw_variant = get_raw_variant(&path).unwrap();
        assert_eq!("RAF", raw_variant.extension().unwrap())
    }
}
