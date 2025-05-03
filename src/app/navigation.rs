use super::*;

impl BlitzApp {
    pub fn handle_user_input(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Open folder…").clicked() {
            // save_culling_progress(&self.photo_dir, photos);

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
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
}

pub fn go_to_next_picture(template_app: &mut BlitzApp) {
    log::info!("Go to next picture");
    if let Ok(photos) = template_app.photos.try_read() {
        match get_next_picture_index(template_app.photos_index, &photos) {
            Some(index) => {
                log::info!("Moving to index: {}", index);
                template_app.photos_index = index;
            }
            None => todo!("we rated everything so now we die"),
        }
    }
}

pub fn go_to_previous_picture(template_app: &mut BlitzApp) {
    if let Ok(photos) = template_app.photos.try_read() {
        match get_previous_picture_index(template_app.photos_index, &photos) {
            Some(index) => template_app.photos_index = index,
            None => todo!("we rated everything so now we die"),
        }
    }
}

pub fn get_next_picture_index(starting_index: usize, photos: &[ImageInfo]) -> Option<usize> {
    log::info!("Get next picture index");
    let mut candidate_index = starting_index;
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

pub fn get_previous_picture_index(starting_index: usize, photos: &[ImageInfo]) -> Option<usize> {
    let mut candidate_index = starting_index;
    loop {
        if candidate_index == 0 {
            candidate_index = photos.len() - 1;
        } else {
            candidate_index -= 1;
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
}
