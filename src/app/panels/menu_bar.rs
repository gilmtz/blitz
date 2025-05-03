use crate::BlitzApp;

impl BlitzApp {
    pub fn setup_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // NOTE: no File->Quit on web pages!
        let is_web = cfg!(target_arch = "wasm32");
        if !is_web {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(10.0);

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Choose Wheat Dir").clicked() {
                    self.wheat_dir_target = rfd::FileDialog::new().pick_folder();
                    log::debug!("Chose {:?} as wheat directory", self.chaffe_dir_target);
                    ui.close_menu();
                }

                ui.add_space(10.0);

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Choose Chaffe Dir").clicked() {
                    self.chaffe_dir_target = rfd::FileDialog::new().pick_folder();
                    log::debug!("Chose {:?} as chaffe directory", self.chaffe_dir_target);
                    ui.close_menu();
                }
            });
            ui.add_space(16.0);
        }

        egui::widgets::global_theme_preference_buttons(ui);
    }
}
