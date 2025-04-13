use std::{
    ffi::OsString,
    fs::{self},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use egui::{ColorImage, TextureHandle};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;
use web_sys::{console, js_sys::{self, ArrayBuffer, AsyncIterator, Promise, Uint8Array}, window, DirectoryPickerOptions, File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemHandle, FileSystemHandleKind};

use super::{BlitzApp, ImageInfo, Rating};

impl BlitzApp {
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

    pub fn open_folder_action_wasm(&mut self, ui: &mut egui::Ui) -> Result<(), JsValue> {
        // self.open_directory()
        wasm_bindgen_futures::spawn_local(async move {
            let window = web_sys::window().expect("should still have a window");
            match window.show_directory_picker() {
                Ok(promise) => {
                    console::info_1(&"showDirectoryPicker called, awaiting promise...".into());
                    match JsFuture::from(promise).await {
                        Ok(result_value) => {
                            console::info_1(&"Promise resolved!".into());
                            match result_value.dyn_into::<FileSystemDirectoryHandle>() {
                                Ok(handle) => {
                                    console::info_1(&"Got directory handle!".into());
                                    Self::process_directory(handle).await;
                                    console::info_1(&"Got directory handle!".into());
                                }
                                Err(e) => {
                                    console::error_1(&format!("Failed to cast promise result to DirectoryHandle: {:?}", e).into());
                                }
                            }
                        }
                        Err(e) => {
                            // This catches errors like the user cancelling the picker
                            console::error_1(&format!("Failed to await directory picker promise: {:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    // This catches errors if showDirectoryPicker fails immediately (e.g., not supported, security context)
                     console::error_1(&format!("Failed to call showDirectoryPicker: {:?}", e).into());
                }
            }
        });

        Ok(())
    }

    pub async fn process_directory(dir_handle: FileSystemDirectoryHandle) -> Result<(), JsValue> {
        console::log_1(&format!("Processing directory: {}", dir_handle.name()).into());

        // 1. Get the asynchronous iterator for the directory entries (values)
        //    values() gives FileSystemHandle instances
        let iterator: AsyncIterator = dir_handle.values();
    
        loop {
            // 2. Get the next item from the iterator
            //    We need to manually drive the async iterator using its next() method
            let next_promise = iterator.next()?; // Returns a Promise<IteratorResult>
            let next_result = JsFuture::from(next_promise).await?; // Await the promise
    
            // Check if the iterator is done
            let is_done = js_sys::Reflect::get(&next_result, &"done".into())?.as_bool().unwrap_or(true);
            if is_done {
                break; // Exit loop when done
            }
    
            // Get the value (FileSystemHandle)
            let value = js_sys::Reflect::get(&next_result, &"value".into())?;
            let handle: FileSystemHandle = value.dyn_into()?;
    
            // 3. Check the kind (file or directory)
            match handle.kind() {
                FileSystemHandleKind::File => {
                    // 4. It's a file, cast to FileSystemFileHandle
                    match handle.dyn_into::<FileSystemFileHandle>() {
                        Ok(file_handle) => {
                            // Process the file handle
                            if let Err(e) = Self::process_file(file_handle).await {
                               console::error_1(&format!("Error processing file: {:?}", e).into());
                            }
                        }
                        Err(e) => {
                            console::error_1(&format!("Error casting to FileSystemFileHandle: {:?}", e).into());
                        }
                    }
                }
                FileSystemHandleKind::Directory => {
                    // It's a directory, cast to FileSystemDirectoryHandle
                     match handle.dyn_into::<FileSystemDirectoryHandle>() {
                        Ok(sub_dir_handle) => {
                            console::log_1(&format!("Found subdirectory: {}", sub_dir_handle.name()).into());
                            // --- Optional: Recurse into subdirectory ---
                            // if let Err(e) = process_directory(sub_dir_handle).await {
                            //     console::error_1(&format!("Error processing subdirectory: {:?}", e).into());
                            // }
                        }
                        Err(e) => {
                           console::error_1(&format!("Error casting to FileSystemDirectoryHandle: {:?}", e).into());
                        }
                     }
                }
                _ => {
                    // Handle potential other kinds if the API evolves
                    console::warn_1(&"Found handle of unknown kind".into());
                }
            }
        }
    
        console::log_1(&format!("Finished processing directory: {}", dir_handle.name()).into());
        Ok(())   
    }

    async fn process_file(file_handle: FileSystemFileHandle) -> Result<(), JsValue> {
        console::log_1(&format!("Processing file: {}", file_handle.name()).into());
    
        // 5. Get the File object from the handle
        let file_promise = file_handle.get_file(); // Returns Promise<File>
        let file_obj: File = JsFuture::from(file_promise).await?.dyn_into()?;
    
        // 6. Read the file contents as an ArrayBuffer
        let buffer_promise = file_obj.array_buffer(); // Returns Promise<ArrayBuffer>
        let array_buffer: ArrayBuffer = JsFuture::from(buffer_promise).await?.dyn_into()?;
    
        // 7. Convert ArrayBuffer to Rust bytes (Vec<u8>)
        //    Create a Uint8Array view onto the ArrayBuffer
        let byte_array = Uint8Array::new(&array_buffer);
        //    Copy the data into a Rust Vec<u8>
        let bytes: Vec<u8> = byte_array.to_vec();
    
        console::log_1(&format!("Read {} bytes from file: {}", bytes.len(), file_handle.name()).into());
    
        // ---->>>> TODO: DO SOMETHING WITH THE FILE NAME and `bytes` VEC <<<<----
        // For example, store them, display them, parse them, etc.
    
        Ok(())
    }
}

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
