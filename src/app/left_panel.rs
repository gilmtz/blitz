use std::
    sync::{Arc, RwLock}
;

use egui::ImageSource;

use super::{BlitzApp, ImageInfo, Rating};

impl BlitzApp {
    pub fn update_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Queue");

            egui::ScrollArea::vertical().show(ui, |ui| {
                // if let Ok(photos) = (&self.photos).try_read() {
                //     for (index, photo) in photos.iter().enumerate() {
                //         self.render_photo_item(photo, ui, index);
                //     }
                // } 
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
            Rating::Unrated => self.render_unrated_photo(photo, ui, index),
            Rating::Approve => {}
            Rating::Remove => {}
        }
    }

    fn render_unrated_photo(
        &mut self,
        photo: &Arc<RwLock<ImageInfo>>,
        ui: &mut egui::Ui,
        index: usize,
    ) {
        if let Ok(image_info) = photo.try_read(){
            self.display_thumbnail(ui, index, &image_info);
        }
    }

    fn display_thumbnail(
        &mut self, 
        ui: &mut egui::Ui, 
        index: usize, 
        photo: &ImageInfo, 
    ) {
        if let Ok(texture_handle_guard) = (&photo.texture).try_lock() {
            let image_source:ImageSource<'_> = match *texture_handle_guard {
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

            let image_widget = ui.add(image);
            if image_widget.clicked() {
                self.photos_index = index
            }
            image_widget.context_menu(|ui| {
                // context_menu::add_open_file_location_option(photo, ui);
                // context_menu::add_open_file_option(&owned_photo, ui);
            });
            ui.label(photo.image_name.clone());
        };
        
        
    }
    
}
