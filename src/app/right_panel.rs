use super::{Rating, BlitzApp};

use std::
    sync::{Arc, RwLock, RwLockReadGuard}
;

impl BlitzApp {
    pub fn update_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Keep");

            for photo in self.photos.iter().rev() {
                render_photo_image(photo, ui);
            }
        });
    }
}

fn render_photo_image(photo: &Arc<RwLock<super::ImageInfo>>, ui: &mut egui::Ui) {
    let current_image = photo.read().unwrap();
    match current_image.rating {
        Rating::Unrated => {}
        Rating::Approve => {
            handle_approve_image(photo, ui, current_image);
        }
        Rating::Remove => {}
    }
}

fn handle_approve_image(photo: &Arc<RwLock<super::ImageInfo>>, ui: &mut egui::Ui, current_image: RwLockReadGuard<'_, super::ImageInfo>) {
    if let Ok(texture_handle) = photo.read().unwrap().texture.clone().try_lock() {
        let texture = texture_handle.as_ref();
        let image = match texture {
            Some(texture) => egui::Image::new(texture).max_width(100.0),
            None => egui::Image::new("file://assets/icon-1024.png").max_width(100.0),
        };
        ui.add(image);
        ui.label(current_image.image_name.clone());
    }
}