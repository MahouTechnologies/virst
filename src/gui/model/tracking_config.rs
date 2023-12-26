use std::{
    hash::Hash,
    mem::{discriminant, take},
    sync::Arc,
};

use egui::{CollapsingHeader, ComboBox, Context, DragValue, Ui};
use inox2d::{model::Model, puppet::Puppet};

use crate::{displayed_model::DisplayedModel, tracker::*};

pub struct TrackingConfig {
    pub open: bool,
    displayed_model: Arc<DisplayedModel>,
    binding: ParamBindings,
    current: u32,
    model: Option<Arc<Model>>,
}

fn bindings_for_model(model: &Puppet) -> ParamBindings {
    let mut out = ParamBindings::new();
    for (k, v) in &model.parameters {
        if v.is_vec2 {
            out.insert(k.to_owned(), ParamBinding::TwoDim(None));
        } else {
            out.insert(k.to_owned(), ParamBinding::OneDim(None));
        }
    }
    out
}

impl TrackingConfig {
    pub fn new(displayed_model: Arc<DisplayedModel>) -> Self {
        let (model, current) = displayed_model.current_model();
        Self {
            open: false,
            displayed_model,
            binding: if let Some(model) = &model {
                bindings_for_model(&model.puppet)
            } else {
                ParamBindings::new()
            },
            current,
            model,
        }
    }

    pub fn draw(&mut self, ctx: &Context, tracker_system: &TrackerSystem) {
        let (model, current) = self.displayed_model.current_model();
        if current > self.current {
            self.current = current;
            self.binding = if let Some(model) = &model {
                bindings_for_model(&model.puppet)
            } else {
                ParamBindings::new()
            };
            self.model = model;
        } else {
            let mut locked = self.displayed_model.bindings.lock().unwrap();
            self.binding = take(&mut locked);
        }

        egui::Window::new("Tracking Config")
            .open(&mut self.open)
            .show(ctx, |ui| {
                if self.model.is_some() {
                    let (collapse, expand) = ui
                        .horizontal(|ui| {
                            let collapse = ui.button("Collapse All").clicked();
                            let expand = ui.button("Expand All").clicked();
                            (collapse, expand)
                        })
                        .inner;

                    // If both happen at once, which should be impossible,
                    // just collapse them all.
                    let open = if collapse {
                        Some(false)
                    } else if expand {
                        Some(true)
                    } else {
                        None
                    };

                    ui.separator();

                    ui.vertical(|ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                Self::binding_toggles(
                                    ui,
                                    self.current,
                                    open,
                                    tracker_system,
                                    &mut self.binding,
                                );
                            });
                    });
                } else {
                    ui.label("No Model Shown");
                }
            });

        let mut locked = self.displayed_model.bindings.lock().unwrap();
        *locked = take(&mut self.binding);
    }

    fn binding_toggles(
        ui: &mut Ui,
        current: u32,
        open: Option<bool>,
        tracker_system: &TrackerSystem,
        binding: &mut ParamBindings,
    ) {
        let data = tracker_system.data().lock().unwrap();
        let mut possible_bindings = vec![InputKind::None];
        for (k, v) in &data.blends {
            possible_bindings.push(InputKind::Blendshape(k.clone()));
        }
        for (k, v) in &data.bones {
            use InputBoneKind::*;
            for i in [X, Y, Z, Pitch, Yaw, Roll] {
                possible_bindings.push(InputKind::Bone(k.clone(), i));
            }
        }
        drop(data);

        for (name, binding) in binding {
            let header = CollapsingHeader::new(name)
                .id_source((current, name))
                .open(open)
                .default_open(true);

            let remove = header
                .show(ui, |ui| {
                    if binding.is_bound() {
                        match binding {
                            ParamBinding::OneDim(v) => Self::one_dim_edit(
                                ui,
                                name,
                                v.as_mut().unwrap(),
                                &possible_bindings,
                            ),
                            ParamBinding::TwoDim(v) => Self::two_dim_edit(
                                ui,
                                name,
                                v.as_mut().unwrap(),
                                &possible_bindings,
                            ),
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button("Add Binding").clicked() {
                                binding.default_binding();
                            }
                        });

                        // Cannot remove binding if it doesn't exist
                        false
                    }
                })
                .body_returned;

            if let Some(true) = remove {
                binding.clear_binding();
            }
        }
    }

    fn single_binding_edit<H: Hash + Copy>(
        ui: &mut Ui,
        id_source: H,
        binding: &mut BindingKind,
        input_kinds: &[InputKind],
    ) {
        ui.horizontal(|ui| {
            let selected = match binding {
                BindingKind::Simple { .. } => "Standard",
                BindingKind::Expr => "Expression",
            };

            let disc_to_compare = discriminant(binding);

            ui.label("Binding Type:");

            let simple_disc = discriminant(&BindingKind::simple());
            let expr_disc = discriminant(&BindingKind::expr());

            ComboBox::from_id_source(id_source)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(disc_to_compare == simple_disc, "Standard")
                        .clicked()
                    {
                        *binding = BindingKind::simple();
                    }

                    if ui
                        .selectable_label(disc_to_compare == expr_disc, "Expression")
                        .clicked()
                    {
                        *binding = BindingKind::expr();
                    }
                });
        });

        match binding {
            BindingKind::Expr => {
                ui.label("Not implemented!");
            }
            BindingKind::Simple {
                input,
                input_range,
                output_range,
                dampen,
            } => {
                ComboBox::from_id_source(id_source)
                    .width(300.0)
                    .selected_text(input.name())
                    .show_ui(ui, |ui| {
                        for i in input_kinds {
                            ui.selectable_value(input, i.clone(), i.name());
                        }
                    });

                ui.label("Input Range:");
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut input_range.0).speed(0.1));
                    ui.add(DragValue::new(&mut input_range.1).speed(0.1));
                });

                ui.label("Output Range:");
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut output_range.0).speed(0.1));
                    ui.add(DragValue::new(&mut output_range.1).speed(0.1));
                });
            }
        }
    }

    fn one_dim_edit(
        ui: &mut Ui,
        name: &str,
        binding: &mut BindingKind,
        input_kinds: &[InputKind],
    ) -> bool {
        if ui.button("Remove Binding").clicked() {
            return true;
        }

        ui.separator();

        Self::single_binding_edit(ui, name, binding, input_kinds);

        false
    }

    fn two_dim_edit(
        ui: &mut Ui,
        name: &str,
        bindings: &mut (BindingKind, BindingKind),
        input_kinds: &[InputKind],
    ) -> bool {
        if ui.button("Remove Binding").clicked() {
            return true;
        }

        ui.separator();

        ui.label("X Binding");
        Self::single_binding_edit(ui, (name, "X"), &mut bindings.0, input_kinds);

        ui.separator();

        ui.label("Y Binding");
        Self::single_binding_edit(ui, (name, "Y"), &mut bindings.1, input_kinds);

        false
    }
}
