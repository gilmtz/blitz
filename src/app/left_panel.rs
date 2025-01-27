use std::
    sync::{Arc, RwLock}
;

use egui::{Image, ImageSource};

use super::{context_menu, BlitzApp, ImageInfo, Rating};

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
                let image_source:ImageSource<'_> = match *texture_handle {
                    Some(ref texture) => texture.into(),
                    None => "file://assets/icon-1024.png".into(),
                };
                let image = egui::Image::new(image_source)
                    .max_width(100.0)
                    .sense(egui::Sense {
                        click: true,
                        drag: false,
                        focusable: false,
                    });
                self.render_photo_image(image, ui, index, &owned_photo)
            }
            Err(_) => {}
        };
    }

    fn render_photo_image(
        &mut self,
        image: Image<'_>,
        ui: &mut egui::Ui,
        index: usize,
        owned_photo: &std::sync::RwLockReadGuard<'_, ImageInfo>,
    ) {
        
        let image_widget = ui.add(image);
        if image_widget.clicked() {
            self.photos_index = index
        }
        image_widget.context_menu(|ui| {
            context_menu::add_open_file_location_option(&owned_photo, ui);
            context_menu::add_open_file_option(&owned_photo, ui);
        });
        ui.label(owned_photo.image_name.clone());
    }
}

fn fun_name(ui: &mut egui::Ui, image: Image<'_>) -> egui::Response {
    let image_widget = ui.add(image);
    image_widget
}
