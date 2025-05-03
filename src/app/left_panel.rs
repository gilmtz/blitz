use std::sync::Arc;

use egui::ImageSource;

use super::{BlitzApp, ImageInfo, Rating};

impl BlitzApp {
    pub fn update_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Queue");

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(photos) = self.photos.try_read() {
                    for (index, photo) in photos.iter().enumerate() {
                        render_photo_item(photo, ui, index, &mut self.photos_index);
                    }
                }
            });
        });
    }
}

fn render_photo_item(photo: &ImageInfo, ui: &mut egui::Ui, index: usize, photos_index: &mut usize) {
    let owned_photo = photo;
    match owned_photo.rating {
        Rating::Unrated => render_unrated_photo(photo, ui, index, photos_index),
        Rating::Approve => {}
        Rating::Remove => {}
    }
}

fn render_unrated_photo(
    photo: &ImageInfo,
    ui: &mut egui::Ui,
    index: usize,
    photos_index: &mut usize,
) {
    display_thumbnail(ui, index, &photo, photos_index);
}

fn display_thumbnail(
    ui: &mut egui::Ui,
    index: usize,
    photo: &ImageInfo,
    app_photo_index: &mut usize,
) {
    if let Ok(texture_handle_guard) = (&photo.texture).try_lock() {
        let image_source: ImageSource<'_> = match *texture_handle_guard {
            Some(ref texture) => texture.into(),
            None => {
                // "file://assets/icon-1024.png".into()
                let bytes: Arc<[u8]> = photo.data.clone();
                let byte_path = format!("bytes://{}", photo.image_name);
                ImageSource::Bytes {
                    uri: byte_path.into(),
                    bytes: egui::load::Bytes::Shared(bytes),
                }
            }
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
            *app_photo_index = index.clone()
        }
        image_widget.context_menu(|_ui| {
            // context_menu::add_open_file_location_option(photo, ui);
            // context_menu::add_open_file_option(&owned_photo, ui);
        });
        ui.label(photo.image_name.clone());
    };
}
