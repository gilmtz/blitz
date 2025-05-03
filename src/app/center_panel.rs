use std::sync::Arc;

use super::{models::ImageInfo, BlitzApp};

use egui::{Color32, Vec2};
#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileHandle;

impl BlitzApp {
    pub fn update_center_panel(&mut self, ctx: &egui::Context) {
        let photos_index = self.photos_index;

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("blitz");
            ui.add(
                egui::Slider::new(&mut self.max_texture_count, 0..=500).text("Max Texture Count"),
            );
            self.handle_user_input(ctx, ui);


            if let Ok(photos) = self.photos.try_read() {
                if !photos.is_empty() {
                    if let Some(current_image) = photos.get(photos_index) {
                        let max_height = ui.available_height();
                            let max_width = ui.available_width();
    
                            display_image(max_width, max_height, ui, ctx, current_image);   
                    }
                }
            }


            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/gilmtz/blitz/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                Self::powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }


    #[cfg(not(target_arch = "wasm32"))]
    fn _hot_load_image(
        &mut self,
        file_handle: FileHandle,
        max_width: f32,
        max_height: f32,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        current_image: &super::ImageInfo,
    ) -> egui::Response {
        let bytes: Arc<[u8]> = block_on(file_handle.read()).into();
        // let uri = format!("bytes://{}", current_image.image_name);
        let uri = "bytes://asdf.jpeg";
        // let image_source = egui::Image::from_bytes(uri, bytes);
        let image = egui::Image::from_bytes(uri, bytes)
            .max_width(max_width)
            .max_height(max_height)
            .sense(egui::Sense {
                click: false,
                drag: true,
                focusable: false,
            });
        let image_widget = ui.add(image);
        // vec2
        if image_widget.dragged() {
            // image.uv(egui::Rect {min:  [0.0, 0.0].into(), max: [0.5, 0.5].into()});
            println!("Image dragged");
        }
        // if image_widget.hovered() {
        //     self.handle_hover_action(ctx, image_widget, texture);
        //     // println!("{}", image_widget.rect);
        // }
        ui.label(current_image.image_name.clone())
    }

    fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Powered by ");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label(" and ");
            ui.hyperlink_to(
                "eframe",
                "https://github.com/emilk/egui/tree/master/crates/eframe",
            );
            ui.label(".");
        });
    }
}

fn display_image(
    max_width: f32,
    max_height: f32,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    current_image: &ImageInfo,
) -> egui::Response {
    let texture_guard = current_image.texture.try_lock();

    match texture_guard {
        Ok(texture_guard) => {
            match &*texture_guard {
                Some(texture) => {
                    let image = egui::Image::new(texture)
                    .max_width(max_width)
                    .max_height(max_height)
                    .sense(egui::Sense {
                        click: false,
                        drag: true,
                        focusable: false,
                    });
                    let image_widget = ui.add(image);
                    // vec2
                    if image_widget.dragged() {
                        // image.uv(egui::Rect {min:  [0.0, 0.0].into(), max: [0.5, 0.5].into()});
                        println!("Image dragged");
                    }
                    if image_widget.hovered() {
                        handle_hover_action(ctx, image_widget, texture);

                        // println!("{}", image_widget.rect);
                    }
                    ui.label(current_image.image_name.clone())
                },
                None => {
                    display_image_with_bytes(max_width, max_height, ui, ctx, current_image)
                },
            }
            
        },
        Err(_) => todo!("didn't handle the err in display image"),
    }     
}


fn display_image_with_bytes(
    max_width: f32,
    max_height: f32,
    ui: &mut egui::Ui,
    _ctx: &egui::Context,
    current_image: &ImageInfo,
) -> egui::Response {

    let bytes: Arc<[u8]> = current_image.data.clone();
    let byte_path = format!("bytes://{}", current_image.image_name);
    let image = egui::Image::from_bytes(byte_path, bytes)
        .max_width(max_width)
        .max_height(max_height);
    let image_widget = ui.add(image);
    return image_widget;
}

fn handle_hover_action(
    ctx: &egui::Context,
    image_widget: egui::Response,
    texture: &egui::TextureHandle,
) {
    // Draw the image at the cursor position
    if let Some(pos) = ctx.pointer_interact_pos() {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("cursor_layer"),
        ));
        let image_size = Vec2::new(300.0, 300.0); // Adjust size as needed
        let pos_rect = egui::Rect::from_min_max(pos, pos + image_size);

        let relative_pos = pos - image_widget.rect.min;
        let normalized_x = relative_pos.x / image_widget.rect.width();
        let normalized_y = relative_pos.y / image_widget.rect.height();

        let zoom_level = egui::vec2(0.03, 0.03);

        let normalized_pos = egui::pos2(normalized_x, normalized_y);

        let uv = egui::Rect::from_min_max(normalized_pos, normalized_pos + zoom_level);
        painter.add(egui::Shape::image(
            texture.id(),
            pos_rect,
            uv,
            Color32::WHITE,
        ));
    }
}

fn _display_placeholder(ui: &mut egui::Ui, current_image: super::ImageInfo) -> egui::Response {
    ui.add(egui::Image::new("file://assets/icon-1024.png").max_width(1500.0));
    ui.label(current_image.image_name.clone())
}
