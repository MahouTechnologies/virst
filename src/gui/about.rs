use egui::special_emojis::GITHUB;
use egui::Context;

pub struct About;

impl About {
    pub fn draw(open: &mut bool, ctx: &Context) {
        egui::Window::new("About").open(open).show(ctx, |ui| {
            ui.heading("Virst");
            ui.label("VIRST (styled Virst), the Virtual Integrated Rendering System for Talents, is an streaming app for virtual talents and VTubers. \
            Virst aims to be a powerful app that integrates everything needed for streaming activties, including models, items, scenes, and more!");

            ui.add_space(12.0);
            ui.heading("Model Support");
            ui.label("Virst currently only supports Inochi2D models. In the future, other 2D and maybe 3D model formats will be supported.");

            ui.add_space(12.0);
            ui.heading("Links");
            ui.label("You may find me on GitHub. Issues are best reported TODO.");
            ui.hyperlink_to(
                format!("{} carbotaniuman", GITHUB),
                "https://github.com/carbotaniuman",
            );
        });
    }
}
