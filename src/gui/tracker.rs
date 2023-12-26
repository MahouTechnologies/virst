use std::{net::IpAddr, str::FromStr};

use egui::{Button, Context, TextEdit};

use crate::tracker::TrackerSystem;

#[derive(Default)]
pub struct Tracker {
    pub open: bool,
    ip_string: String,
    ip_dirty: bool,
    port_string: String,
    port_dirty: bool,
}

impl Tracker {
    pub fn draw(&mut self, ctx: &Context, tracker_system: &mut TrackerSystem) {
        egui::Window::new("Tracker")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("Connection Status:");

                ui.separator();

                let ip_response = ui
                    .horizontal(|ui| {
                        let error = !self.ip_string.is_empty()
                            && IpAddr::from_str(&self.ip_string).is_err();
                        ui.label("IP Address:");
                        let response = ui
                            .add(TextEdit::singleline(&mut self.ip_string).hint_text("127.0.0.1"));

                        (error, response.lost_focus())
                    })
                    .inner;
                if ip_response.1 {
                    self.ip_dirty = true;
                }
                ui.label(if ip_response.0 && self.ip_dirty {
                    "Invalid IP address!"
                } else {
                    ""
                });

                let port_response = ui
                    .horizontal(|ui| {
                        let error = !self.port_string.is_empty()
                            && u16::from_str(&self.port_string).is_err();
                        ui.label("Port:");
                        let response =
                            ui.add(TextEdit::singleline(&mut self.port_string).hint_text("39539"));

                        (error, response.lost_focus())
                    })
                    .inner;
                if port_response.1 {
                    self.port_dirty = true;
                }
                ui.label(if port_response.0 && self.port_dirty {
                    "Invalid port number!"
                } else {
                    ""
                });

                let valid = !self.ip_string.is_empty()
                    && !self.port_string.is_empty()
                    && !ip_response.0
                    && !port_response.0;
                let clicked = ui
                    .add_enabled(valid, Button::new("Force Reconnect"))
                    .clicked();
                let auto_refresh = ip_response.1 || port_response.1;

                if valid && (clicked || auto_refresh) {
                    let ip = IpAddr::from_str(&self.ip_string).unwrap();
                    let port = u16::from_str(&self.port_string).unwrap();
                    println!("{:?} {}", ip, port);
                    tracker_system.disconnect();
                    let _ = tracker_system.connect((ip, port));
                }
            });
    }
}
