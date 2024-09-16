#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::{fs, path::PathBuf, sync::{Arc, Mutex, RwLock}};

pub use app::TemplateApp;

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


fn commit_culling(photos: &Vec<Arc<RwLock<ImageInfo>>>, root_dir: PathBuf) {
    let mut chaffe_dir = root_dir.clone();
    chaffe_dir.push("chaffe");
    let mut wheat_dir = root_dir.clone();
    wheat_dir.push("wheat");

    match fs::create_dir_all(chaffe_dir.clone()) {
        Ok(it) => it,
        Err(_err) => todo!("handle when we can't make directories later"),
    };

    match fs::create_dir_all(wheat_dir.clone()) {
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

    println!("{}", image.path_raw.clone().unwrap().display());
    match &image.path_raw {
        Some(path_raw) => {
            let mut raw_image_destination = destination_dir.clone();
            raw_image_destination.push(image.image_name.clone());
            raw_image_destination.set_extension("RAF");
            let _ = fs::copy(path_raw.clone(), raw_image_destination);
        },
        None => {},
    }
    
}

fn get_raw_variant(processed_path: &PathBuf) -> Option<PathBuf> {
    let mut raw_path = processed_path.clone();
    match raw_path.set_extension("RAF"){
        true => Some(raw_path),
        false => None,
    }
        
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_raw_variant() {
        let path = PathBuf::from("/tmp/DSC55555.jpg");
        let raw_variant = get_raw_variant(&path).unwrap();
        assert_eq!("RAF",raw_variant.extension().unwrap())
    }
}