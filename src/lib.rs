#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::{fs, path::PathBuf, sync::{Arc, Mutex, RwLock}};

pub use app::TemplateApp;
use ron::ser::PrettyConfig;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone)]
pub struct ImageInfo {
    path_processed: PathBuf,
    path_raw: Option<PathBuf>,
    rating: Rating,
    #[serde(skip)]
    texture: Arc<Mutex<Option<egui::TextureHandle>>>,
    image_name: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
enum Rating {
    Skip,
    Approve,
    Remove,
}


fn commit_culling(photos: &Vec<Arc<RwLock<ImageInfo>>>, root_dir: PathBuf, dry_run_mode: bool) {
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
                copy_image_into_dir(&wheat_dir, &image.read().unwrap());
                if !dry_run_mode {
                    delete_image(&image.read().unwrap());
                }
            }
            Rating::Remove => {
                copy_image_into_dir(&chaffe_dir, &image.read().unwrap());
                if !dry_run_mode {
                    delete_image(&image.read().unwrap());
                }
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

fn delete_image(image: &ImageInfo) -> std::io::Result<()>{
    fs::remove_file(image.path_processed.clone())?;

    match &image.path_raw {
        Some(path_raw) => {
            fs::remove_file(path_raw.clone())?;
        },
        None => {},
    }
    Ok(())
}

fn save_culling_progress(photo_dir: &PathBuf, photos: &Vec<Arc<RwLock<ImageInfo>>>){
    if photos.len()  < 1 {
        return;
    }
    let mut blitz_dir = photo_dir.clone();
    blitz_dir.push(".blitz");

    match fs::create_dir_all(blitz_dir.clone()) {
        Ok(_dir) => {}
        Err(_err) => {}
    };

    blitz_dir.push("storage.ron");

    let ron_str = ron::ser::to_string_pretty(&photos, PrettyConfig::new());
    match ron_str {
        Ok(serialized_ron) => {
            let _ = fs::write(blitz_dir, serialized_ron);
        }
        Err(_) => {
            todo!("serializing didn't work")
        }
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