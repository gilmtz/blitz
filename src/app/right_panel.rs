use super::{context_menu, models::ImageInfo, BlitzApp, Rating};

use std::
    sync::{Arc, RwLock, RwLockReadGuard}
;

impl BlitzApp {
    pub fn update_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Keep");

            if let Ok(photos) = self.photos.try_read() {
                for photo in photos.iter().rev() {
                    render_photo_image(photo, ui);
                }
            }
        });
    }
    
}

fn render_photo_image(current_image: &ImageInfo, ui: &mut egui::Ui) {
    match current_image.rating {
        Rating::Unrated => {}
        Rating::Approve => {
            handle_approve_image(current_image, ui);
        }
        Rating::Remove => {}
    }
}

fn handle_approve_image(photo: &ImageInfo, ui: &mut egui::Ui) {
    if let Ok(texture_handle) = photo.texture.try_lock() {
        let texture = texture_handle.as_ref();
        let image = match texture {
            Some(texture) => egui::Image::new(texture).max_width(100.0),
            None => {
                let bytes: Arc<[u8]> = photo.data.clone();
                let byte_path = format!("bytes://{}", photo.image_name);
                egui::Image::from_bytes(byte_path, bytes).max_height(100.0)
            },
        };
        let image_widget = ui.add(image);
        image_widget.context_menu(|ui| {
            context_menu::add_open_file_location_option(photo.clone(), ui);
            context_menu::add_open_file_option(photo.clone(), ui);
        });

        ui.label(photo.image_name.clone());
    }
}
