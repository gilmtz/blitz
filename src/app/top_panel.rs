use super::*;

impl BlitzApp {

    pub fn update_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                self.setup_menu_bar(ctx, ui);
            });
        });
    }
    
}