use super::BlitzApp;

impl BlitzApp {
    pub fn update_center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("blitz");
            ui.add(
                egui::Slider::new(&mut self.max_texture_count, 0..=500).text("Max Texture Count"),
            );

            self.handle_user_input(ctx, ui);

            if self.photos.len() > 0 {
                let current_image = self.photos[self.photos_index].read().unwrap().clone();
                let max_height = ui.available_height();
                let max_width = ui.available_width();

                if let Ok(texture_handle) = current_image.texture.clone().try_lock() {
                    match *texture_handle {
                        Some(ref texture) => self.display_image(texture, max_width, max_height, ui, ctx, &current_image),
                        None => display_placeholder(ui, current_image),
                    };
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

    fn display_image(&mut self, texture: &egui::TextureHandle, max_width: f32, max_height: f32, ui: &mut egui::Ui, ctx: &egui::Context, current_image: &super::ImageInfo) -> egui::Response {
        let image = egui::Image::new(texture)
            .max_width(max_width)
            .max_height(max_height)
            .sense(egui::Sense {
                click: false,
                drag: true,
                focusable: false,
            })
            .uv(egui::Rect {
                min: self.uv_min,
                max: self.uv_max,
            });
        let image_widget = ui.add(image);
        // vec2
        if image_widget.dragged() {
            // image.uv(egui::Rect {min:  [0.0, 0.0].into(), max: [0.5, 0.5].into()});
            println!("Image dragged");
        }
        if image_widget.hovered() {
            self.handle_hover_action(ctx, image_widget);
            // println!("{}", image_widget.rect);
        }
        ui.label(current_image.image_name.clone())
    }
    
    fn handle_hover_action(&mut self, ctx: &egui::Context, image_widget: egui::Response) {
        // println!("image_widget hover pos: {}", image_widget.hover_pos().unwrap_or([0.0,0.0].into()));
        ctx.input(|i| {
            let scroll_vec = i.raw_scroll_delta;
            let abs_hover_pos =
                i.pointer.hover_pos().unwrap_or([0.0, 0.0].into());
            // println!("i.pointer: {}, rect: {}", i.pointer.hover_pos().unwrap_or([0.0,0.0].into()), image_widget.rect);
            let relative_pos = egui::Pos2 {
                x: abs_hover_pos.x - image_widget.interact_rect.min.x, // TODO: value is sometimes negative at the very edge
                y: abs_hover_pos.y - image_widget.interact_rect.min.y,
            };
    
            let rect_size = egui::Vec2 {
                x: image_widget.rect.max.x - image_widget.rect.min.x,
                y: image_widget.rect.max.y - image_widget.rect.min.y
            };
    
            let uv_min = egui::Pos2 {
                x: f32::max(0.0,f32::min(1.0, (relative_pos.x / rect_size.x) - self.uv_size/2.0)),
                y: f32::max(0.0,f32::min(1.0, (relative_pos.y / rect_size.y) - self.uv_size/2.0)),
            };
            let uv_max = egui::Pos2 {
                x: f32::min(1.0, self.uv_min.x + self.uv_size),
                y: f32::min(1.0, self.uv_min.y + self.uv_size),
            };
            // println!("relative_pos: {}, uv: {}", relative_pos, uv_pos);
            if scroll_vec.angle() != 0.0 {
                println!(
                    "{}, {}, {}",
                    scroll_vec,
                    scroll_vec.length(),
                    scroll_vec.angle()
                );
                self.uv_size = f32::min(
                    1.0,
                    self.uv_size - (scroll_vec.angle() * 0.1),
                );
                self.uv_min = uv_min;
                self.uv_max = uv_max;
            }
        });
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

fn display_placeholder(ui: &mut egui::Ui, current_image: super::ImageInfo) -> egui::Response {
    ui.add(
        egui::Image::new("file://assets/icon-1024.png")
            .max_width(1500.0),
    );
    ui.label(current_image.image_name.clone())
}