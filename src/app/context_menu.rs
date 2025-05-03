use super::models::ImageInfo;

pub fn add_open_file_option(unwrapped_photo: ImageInfo, ui: &mut egui::Ui) {
    
    #[cfg(not(target_arch = "wasm32"))]
    if ui.button("Open file location").clicked() {
        let mut photo_location = unwrapped_photo.path_processed.clone();
        photo_location.pop();
        let _ = open::that(photo_location);
        ui.close_menu();
    }

    #[cfg(target_arch = "wasm32")]
    todo!("implement add_open_file_location")

}

pub fn add_open_file_location_option(unwrapped_photo: ImageInfo, ui: &mut egui::Ui) {
    #[cfg(not(target_arch = "wasm32"))]
    if ui.button("Open file").clicked() {
        let _ = open::that(unwrapped_photo.path_processed.clone());
        ui.close_menu();
    }

    #[cfg(target_arch = "wasm32")]
    todo!("implement add_open_file_location_option")
}