use std::{
    ffi::OsString,
    fs::File,
    future::Future,
    io::BufReader,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::{bounded, Receiver, Sender};
use egui::{Align, Context, Layout, RichText};
use inox2d::{formats::inp::parse_inp, model::Model};

use crate::displayed_model::DisplayedModel;

pub struct ModelManager {
    pub open: bool,
    processing: Arc<AtomicBool>,
    displayed_model: Arc<DisplayedModel>,

    models: Vec<(OsString, Model)>,
    sender: Sender<(OsString, Model)>,
    receiver: Receiver<(OsString, Model)>,
}

impl ModelManager {
    pub fn new(displayed_model: Arc<DisplayedModel>) -> Self {
        let (sender, receiver) = bounded(3);
        Self {
            open: false,
            processing: Arc::new(AtomicBool::new(false)),
            displayed_model,

            models: Vec::new(),
            sender,
            receiver,
        }
    }

    pub fn has_model(&self) -> bool {
        self.displayed_model.current_model().0.is_some()
    }

    pub fn draw(&mut self, ctx: &Context) {
        egui::Window::new("Model Manager")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui: &mut egui::Ui| {
                    if ui
                        .add_enabled(
                            !self.processing.load(Ordering::SeqCst),
                            egui::Button::new("Load Model"),
                        )
                        .clicked()
                    {
                        let task = rfd::AsyncFileDialog::new()
                            .add_filter("Inochi Puppet Files", &["inp"])
                            .pick_file();
                        self.processing.store(true, Ordering::SeqCst);

                        let processing = self.processing.clone();
                        let sender: Sender<(OsString, Model)> = self.sender.clone();
                        execute(async move {
                            let file = task.await;

                            if let Some(file) = file {
                                // TODO: what are errors?
                                let data = file.path();
                                let reader = BufReader::new(File::open(data).unwrap());
                                let model = parse_inp(reader).unwrap();

                                let _ = sender.send((data.file_name().unwrap().to_owned(), model));
                            }
                            processing.store(false, Ordering::SeqCst);
                        });
                    }

                    if ui
                        .add_enabled(
                            self.displayed_model.current_model().0.is_some(),
                            egui::Button::new("Remove Shown Model"),
                        )
                        .clicked()
                    {
                        self.displayed_model.swap_model(None);
                    }
                });

                if let Ok(data) = self.receiver.try_recv() {
                    self.models.push(data);
                }

                let row_height = ui.spacing().interact_size.y;

                if !self.models.is_empty() {
                    ui.separator();

                    let mut to_delete: Option<usize> = None;
                    ui.vertical(|ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show_rows(ui, row_height, self.models.len(), |ui, row_range| {
                                let ind_start = row_range.start;
                                for (ind, (file, model)) in
                                    self.models[row_range].iter().enumerate()
                                {
                                    ui.horizontal_top(|ui| {
                                        ui.label(file.to_string_lossy());
                                        let meta = &model.puppet.meta;

                                        let text = if let Some(name) = &meta.name {
                                            RichText::new(name)
                                        } else {
                                            RichText::new("<unnamed>").italics()
                                        };
                                        ui.label(text);

                                        if ui.button("Show").clicked() {
                                            self.displayed_model.swap_model(Some(
                                                self.models[ind + ind_start].1.clone(),
                                            ));
                                        }

                                        if ui.button("Delete").clicked() {
                                            to_delete = Some(ind + ind_start);
                                        }
                                    });
                                }
                            });
                    });

                    if let Some(ind) = to_delete {
                        self.models.swap_remove(ind);
                    }
                }
            });
    }
}

fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || pollster::block_on(f));
}
