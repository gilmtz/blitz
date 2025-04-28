use std::{
    ffi::OsString,
    fs::{self},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use egui::{ColorImage, TextureHandle};
use futures::{channel::oneshot, executor::block_on};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;
use web_sys::{js_sys::{self, ArrayBuffer, AsyncIterator, Promise, Uint8Array}, window, DirectoryPickerOptions, File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemHandle, FileSystemHandleKind};
use log;

use super::{BlitzApp, ImageInfo, Rating};

pub struct ImageFile {
    pub data: Arc<[u8]>,
    pub name: String,
}


#[cfg(target_arch = "wasm32")]
impl BlitzApp {
    pub fn open_folder_action(&mut self) {
        let image_files = self.photos.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let files = Self::open_folder_action_wasm().await.unwrap_or_else(|err| {
                log::error!("Error opening folder: {:?}", err);
                Vec::new()
            });
            let mut data_guard = image_files.write().unwrap();
            for file in files {

                data_guard.push(ImageInfo{
                    data: file.data,
                    image_name: file.name,
                    path_processed: PathBuf::new(),
                    path_raw: None,
                    rating: Rating::Unrated,
                    texture: Arc::new(Mutex::new(None)),
                }.into());
            }
        });
    }

    pub async fn open_folder_action_wasm() -> Result<Vec<ImageFile>, JsValue> {
        let window = web_sys::window().expect("should still have a window");
        let promise = window.show_directory_picker().map_err(|e| {
            log::error!("Failed to call showDirectoryPicker: {:?}", e);
            e
        })?;
        log::info!("showDirectoryPicker called, awaiting promise...");
        let result_value = JsFuture::from(promise).await?;
        let handle = result_value.dyn_into::<FileSystemDirectoryHandle>().map_err(|e| {
            log::error!("Failed to cast promise result to DirectoryHandle: {:?}", e);
            e
        })?;
        log::info!("Got directory handle!");
        let photos = Self::process_directory(handle).await?;
        log::info!("Processed {} files", photos.len());
        Ok(photos)
    }

    pub async fn process_directory(dir_handle: FileSystemDirectoryHandle) -> Result<Vec<ImageFile>, JsValue> {
        log::debug!("Processing directory: {}", dir_handle.name());

        // 1. Get the asynchronous iterator for the directory entries (values)
        //    values() gives FileSystemHandle instances
        let iterator: AsyncIterator = dir_handle.values();
        let mut photos: Vec<ImageFile> = Vec::new();
    
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
                            let image_file = Self::process_file(file_handle).await;

                            match image_file {
                                Ok(image) => {
                                    log::debug!("Processed file: {}", image.name);
                                    // Here you can do something with the image data
                                    // For example, store it in a vector or process it further
                                    photos.push(image);
                                }
                                Err(e) => {
                                    log::error!("Error processing file: {:?}", e);
                                }
                                
                            }
                        }
                        Err(e) => {
                            log::error!("Error casting to FileSystemFileHandle: {:?}", e);
                        }
                    }
                }
                FileSystemHandleKind::Directory => {
                    // It's a directory, cast to FileSystemDirectoryHandle
                     match handle.dyn_into::<FileSystemDirectoryHandle>() {
                        Ok(sub_dir_handle) => {
                            log::debug!("Found subdirectory: {}", sub_dir_handle.name());
                            // --- Optional: Recurse into subdirectory ---
                            // if let Err(e) = process_directory(sub_dir_handle).await {
                            //     log::error!("Error processing subdirectory: {:?}", e);
                            // }
                        }
                        Err(e) => {
                           log::error!("Error casting to FileSystemDirectoryHandle: {:?}", e);
                        }
                     }
                }
                _ => {
                    // Handle potential other kinds if the API evolves
                    log::warn!("Found handle of unknown kind");
                }
            }
        }
    
        log::debug!("Finished processing directory: {}", dir_handle.name());
        Ok(photos)   
    }

    async fn process_file(file_handle: FileSystemFileHandle) -> Result<ImageFile, JsValue> {
        log::debug!("Processing file: {}", file_handle.name());
    
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
        let bytes: Arc<[u8]> = Arc::from(byte_array.to_vec());
    
        log::debug!("Read {} bytes from file: {}", bytes.len(), file_handle.name());
    
        Ok(ImageFile {
            name: file_handle.name(),
            data: bytes,
        })
    }
}


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_get_raw_variant() {
        assert_eq!(true, true);
    }
}
