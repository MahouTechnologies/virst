use egui::Context;

#[derive(Default)]
pub struct Settings {
    pub open: bool,
}

impl Settings {
    pub fn draw(&mut self, ctx: &Context) {
        egui::Window::new("Settings")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.heading("Data");

                let _ = ui.button("View Stored Data");
                ui.label("Note: Editing or changing the contents of the data folder is not recommended. \
                                This is primarily intended for backup or advanced recovery, and not for loading \
                                models, items, or other assets.");
                ui.separator();

                ui.heading("Plugins");
                let _ = ui.button("Load Plugin");

                ui.label("Current Plugins:")
            });
    }
}
