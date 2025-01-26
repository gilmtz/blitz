use super::{Rating, BlitzApp, context_menu};

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
    let unwrapped_photo = photo.read().unwrap();
    if let Ok(texture_handle) = unwrapped_photo.texture.clone().try_lock() {
        let texture = texture_handle.as_ref();
        let image = match texture {
            Some(texture) => egui::Image::new(texture).max_width(100.0),
            None => egui::Image::new("file://assets/icon-1024.png").max_width(100.0),
        };
        let image_widget = ui.add(image);
        image_widget.context_menu(|ui| {
            context_menu::add_open_file_location_option(&unwrapped_photo, ui);
            context_menu::add_open_file_option(&unwrapped_photo, ui);
        });

        ui.label(current_image.image_name.clone());
    }
}
