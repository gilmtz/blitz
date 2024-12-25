use std::
    sync::{Arc, RwLock}
;

use egui::TextureHandle;

use super::{ImageInfo, Rating, BlitzApp};

impl BlitzApp {
    pub fn update_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Queue");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, photo) in (self.photos.clone()).iter().enumerate() {
                    self.render_photo_item(photo, ui, index);
                }  
            });
        });
    }

    fn render_photo_item(
        &mut self,
        photo: &Arc<RwLock<ImageInfo>>,
        ui: &mut egui::Ui,
        index: usize,
    ) {
        let owned_photo = photo.read().unwrap();
        match owned_photo.rating {
            Rating::Unrated => self.render_unrated_photo(photo, ui, index, owned_photo),
            Rating::Approve => {}
            Rating::Remove => {}
        }
    }

    fn render_unrated_photo(
        &mut self,
        photo: &Arc<RwLock<ImageInfo>>,
        ui: &mut egui::Ui,
        index: usize,
        owned_photo: std::sync::RwLockReadGuard<'_, ImageInfo>,
    ) {
        let texture_mutex = photo.read().unwrap().texture.clone();
        match texture_mutex.try_lock() {
            Ok(texture_handle) => {
                match *texture_handle {
                    Some(ref texture) => self.render_photo_image(texture, ui, index, &owned_photo),
                    None => Self::render_placeholder_image(ui, owned_photo),
                };
            }
            Err(_) => {}
        };
    }

    fn render_placeholder_image(
        ui: &mut egui::Ui,
        owned_photo: std::sync::RwLockReadGuard<'_, ImageInfo>,
    ) {
        ui.add(egui::Image::new("file://assets/icon-1024.png").max_width(100.0));
        ui.label(owned_photo.image_name.clone());
    }

    fn render_photo_image(
        &mut self,
        texture: &TextureHandle,
        ui: &mut egui::Ui,
        index: usize,
        owned_photo: &std::sync::RwLockReadGuard<'_, ImageInfo>,
    ) {
        let image = egui::Image::new(texture)
            .max_width(100.0)
            .sense(egui::Sense {
                click: true,
                drag: true,
                focusable: false,
            });
        let image_widget = ui.add(image);
        if image_widget.clicked() {
            self.photos_index = index
        }
        ui.label(owned_photo.image_name.clone());
    }
}
