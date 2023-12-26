mod about;
mod general_settings;
mod model;
mod tracker;

use std::sync::Arc;

use egui::Context;

use crate::{app::AppState, displayed_model::DisplayedModel, tracker::TrackerSystem};

use self::{
    about::About,
    general_settings::Settings,
    model::{ModelManager, TrackingConfig},
    tracker::Tracker,
};

pub struct Gui {
    show_about: bool,
    model_manager: ModelManager,
    tracking_config: TrackingConfig,
    tracker: Tracker,
    settings: Settings,

    state: AppState,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct GuiResponse {
    pub exited: bool,
}

impl Gui {
    pub fn new(displayed_model: Arc<DisplayedModel>) -> Self {
        Gui {
            show_about: false,
            model_manager: ModelManager::new(displayed_model.clone()),
            tracking_config: TrackingConfig::new(displayed_model),
            tracker: Tracker::default(),
            settings: Settings::default(),

            state: AppState::default(),
        }
    }

    pub fn draw(&mut self, ctx: &Context, tracker_system: &mut TrackerSystem) -> GuiResponse {
        let data = GuiResponse::default();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.toggle_value(&mut self.show_about, "Virst");

                ui.toggle_value(&mut self.settings.open, "Settings");

                ui.toggle_value(&mut self.model_manager.open, "Model Manager");

                ui.toggle_value(&mut self.tracker.open, "Tracker Settings");
            });
        });

        // placing windows here let's us overlap the sidepanel.
        About::draw(&mut self.show_about, ctx);
        self.model_manager.draw(ctx);
        self.tracking_config.draw(ctx, tracker_system);
        self.tracker.draw(ctx, tracker_system);
        self.settings.draw(ctx);

        egui::SidePanel::left("left_panel")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Tracker Status:");
                    ui.label(if tracker_system.active() {
                        "Connected"
                    } else {
                        "Disconnected"
                    });

                    ui.separator();

                    if self.model_manager.has_model() {
                        if ui.button("Model Settings").clicked() {
                            // self.tracking_config.open = !self.tracking_config.open;
                        }

                        if ui.button("Tracking Config").clicked() {
                            self.tracking_config.open = !self.tracking_config.open;
                        }

                        if ui.button("Hotkeys").clicked() {}
                    }
                })
            });

        data
    }
}
