#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

pub use app::TemplateApp;
use ron::ser::PrettyConfig;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ImageInfo {
    path_processed: PathBuf,
    path_raw: Option<PathBuf>,
    rating: Rating,
    #[serde(skip)]
    texture: Arc<Mutex<Option<egui::TextureHandle>>>,
    image_name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
enum Rating {
    Unrated,
    Approve,
    Remove,
}

fn get_chaffe_dir(root_dir: &PathBuf) -> PathBuf {
    let mut chaffe_dir = root_dir.clone();
    chaffe_dir.push("chaffe");
    return  chaffe_dir;
}

fn get_wheat_dir(root_dir: &PathBuf) -> PathBuf {
    let mut wheat_dir = root_dir.clone();
    wheat_dir.push("wheat");
    return wheat_dir;
}

fn commit_culling(
    photos: &Vec<Arc<RwLock<ImageInfo>>>,
    chaffe_dir: &PathBuf,
    wheat_dir: &PathBuf,
    dry_run_mode: bool) {

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
            Rating::Unrated => {}
            Rating::Approve => {
                copy_image_into_dir(&wheat_dir, &image.read().unwrap());
                if !dry_run_mode {
                    let _ = delete_image(&image.read().unwrap());
                }
            }
            Rating::Remove => {
                copy_image_into_dir(&chaffe_dir, &image.read().unwrap());
                if !dry_run_mode {
                    let _ = delete_image(&image.read().unwrap());
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
        }
        None => {}
    }
}

fn delete_image(image: &ImageInfo) -> std::io::Result<()> {
    fs::remove_file(image.path_processed.clone())?;

    match &image.path_raw {
        Some(path_raw) => {
            fs::remove_file(path_raw.clone())?;
        }
        None => {}
    }
    Ok(())
}

fn save_culling_progress(photo_dir: &PathBuf, photos: &Vec<Arc<RwLock<ImageInfo>>>) {
    if photos.len() < 1 {
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
    match raw_path.set_extension("RAF") {
        true => Some(raw_path),
        false => None,
    }
}

fn get_next_picture_index(starting_index: usize, photos: &Vec<Arc<RwLock<ImageInfo>>>) -> Option<usize>{
    let mut candidate_index = starting_index.clone();
    loop {
        candidate_index += 1;
        if candidate_index >= photos.len() {
            candidate_index = 0
        }
        if photos[candidate_index]
            .read()
            .unwrap()
            .rating
            == Rating::Unrated
        {
            return Some(candidate_index);
        }
        if starting_index == candidate_index {
            return None
        }
    }
}

fn get_previous_picture_index(starting_index: usize, photos: &Vec<Arc<RwLock<ImageInfo>>>) -> Option<usize>{
    let mut candidate_index = starting_index.clone();
    loop {
        if candidate_index == 0 {
            candidate_index = photos.len() - 1;
        } else {
            candidate_index = candidate_index - 1;
        }
        if photos[candidate_index]
            .read()
            .unwrap()
            .rating
            == Rating::Unrated
        {
            return Some(candidate_index);
        }
        if starting_index == candidate_index {
            return None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_raw_variant() {
        let path = PathBuf::from("/tmp/DSC55555.jpg");
        let raw_variant = get_raw_variant(&path).unwrap();
        assert_eq!("RAF", raw_variant.extension().unwrap())
    }

    #[test]
    fn test_get_next_picture_index_no_ratings() {
        let mut test_photos = Vec::new();
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));

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
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));

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
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));
        test_photos.push(Arc::new(RwLock::new(ImageInfo {
            path_processed: PathBuf::from("/tmp/DSC55555.jpg"),
            path_raw: Some(PathBuf::from("/tmp/DSC55555.jpg")),
            rating: Rating::Approve,
            texture: Arc::new(Mutex::new(None)),
            image_name: "/tmp/DSC55555.jpg".to_string(),
        })));

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
    fn test_commit_culling(){
        let temp_path = PathBuf::from("tmp");
        fs::create_dir_all(&temp_path).unwrap();

        copy_test_images_to_dir();

        let mut test_photos = Vec::new();
        let image_info = ImageInfo {
            path_processed: PathBuf::from("tmp/1.jpg"),
            path_raw: None,
            rating: Rating::Unrated,
            texture: Arc::new(Mutex::new(None)),
            image_name: "1.jpg".to_string(),
        };
        test_photos.push(Arc::new(RwLock::new(image_info)));

        let temp_path = PathBuf::from("tmp");
        let chaffe_path = PathBuf::from("tmp/chaffe");
        let wheat_path = PathBuf::from("tmp/wheat");

        fs::create_dir_all(&chaffe_path).unwrap();
        fs::create_dir_all(&wheat_path).unwrap();

        commit_culling(&test_photos, &chaffe_path, &wheat_path, false);

        fs::remove_dir_all(&chaffe_path).unwrap();
        fs::remove_dir_all(&wheat_path).unwrap();
        fs::remove_dir_all(&temp_path).unwrap();
    }

    fn copy_test_images_to_dir() {
        fs::copy(PathBuf::from("assets/samples/1.jpg"), PathBuf::from("tmp/1.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/2.jpg"), PathBuf::from("tmp/2.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/3.jpg"), PathBuf::from("tmp/3.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/4.jpg"), PathBuf::from("tmp/4.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/5.jpg"), PathBuf::from("tmp/5.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/6.jpg"), PathBuf::from("tmp/6.jpg")).unwrap();
        fs::copy(PathBuf::from("assets/samples/7.jpg"), PathBuf::from("tmp/7.jpg")).unwrap();
    }
}
