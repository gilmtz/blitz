use super::*;

use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use egui::Key;
use log::{log, Level};
use models::{ImageInfo, Rating};
use ron::ser::PrettyConfig;

use crate::app::navigation::{get_next_picture_index, get_previous_picture_index};

impl BlitzApp {
    #[allow(unused_variables)]
    pub fn commit_choices(&mut self, ui: &mut egui::Ui) {
        match fs::create_dir_all(self.get_chaffe_dir(&(self.photo_dir.clone())).clone()) {
            Ok(it) => it,
            Err(_err) => todo!("handle when we can't make directories later"),
        };

        match fs::create_dir_all(self.get_wheat_dir(&(self.photo_dir.clone())).clone()) {
            Ok(it) => it,
            Err(_err) => todo!("handle when we can't make directories later"),
        };
        let chaffe_dir = &self.get_chaffe_dir(&(self.photo_dir.clone()));
        let wheat_dir = &self.get_wheat_dir(&(self.photo_dir.clone()));
        if let Ok(photos) = self.photos.try_read() {
            commit_culling(&photos, chaffe_dir, wheat_dir);
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.open_folder_action(ui, self.photo_dir.clone());
    }

    fn get_chaffe_dir(&mut self, root_dir: &Path) -> PathBuf {
        match &self.chaffe_dir_target {
            Some(target_dir) => target_dir.clone(),
            None => {
                let mut chaffe_dir = root_dir.to_path_buf();
                chaffe_dir.push("chaffe");
                chaffe_dir
            }
        }
    }

    fn get_wheat_dir(&mut self, root_dir: &Path) -> PathBuf {
        match &self.wheat_dir_target {
            Some(target_dir) => target_dir.clone(),
            None => {
                let mut wheat_dir = root_dir.to_path_buf();
                wheat_dir.push("wheat");
                wheat_dir
            }
        }
    }
}

fn commit_culling(
    photos: &[ImageInfo],
    chaffe_dir: &Path,
    wheat_dir: &Path,
) -> Vec<Result<(), io::Error>> {
    let mut committing_results = vec![];
    for image in photos.iter() {
        handle_image_cull(chaffe_dir, wheat_dir, &mut committing_results, image);
    }
    committing_results
}

fn handle_image_cull(
    chaffe_dir: &Path,
    wheat_dir: &Path,
    committing_results: &mut Vec<Result<(), io::Error>>,
    image: &ImageInfo,
) {
    match image.rating {
        Rating::Unrated => {}
        Rating::Approve => {
            committing_results.push(move_image_into_dir(wheat_dir, image));
        }
        Rating::Remove => {
            committing_results.push(move_image_into_dir(chaffe_dir, image));
        }
    }
}

fn move_image_into_dir(destination_dir: &Path, image: &ImageInfo) -> Result<(), std::io::Error> {
    let mut processed_image_destination = destination_dir.to_path_buf();
    processed_image_destination.push(image.image_name.clone());
    fs::rename(image.path_processed.clone(), processed_image_destination)?;

    if let Some(path_raw) = &image.path_raw {
        let mut raw_image_destination = destination_dir.to_path_buf();
        raw_image_destination.push(image.image_name.clone());
        raw_image_destination.set_extension("RAF");
        fs::rename(path_raw.clone(), raw_image_destination)?;
    }
    Ok(())
}

pub fn save_culling_progress(photo_dir: &Path, photos: &Vec<ImageInfo>) -> io::Result<()> {
    // This handles the initial opening case
    if photos.is_empty() {
        return Ok(());
    }
    let mut blitz_dir = photo_dir.to_path_buf();
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
        let source_bytes = fs::read(src_path_string).unwrap();
        let dest_bytes = fs::read(dest_path_string).unwrap();
        assert_eq!(source_bytes, dest_bytes);
    }
}
