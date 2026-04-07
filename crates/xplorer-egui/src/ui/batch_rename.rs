use std::path::Path;

use eframe::egui;

use crate::theme;

pub struct BatchRenameState {
    pub open: bool,
    pub entries: Vec<RenameEntry>,
    pub find_text: String,
    pub replace_text: String,
    pub mode: RenameMode,
    pub start_number: u32,
    pub prefix: String,
    pub suffix: String,
    pub case_mode: CaseMode,
}

pub struct RenameEntry {
    pub original_path: String,
    pub original_name: String,
    pub new_name: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenameMode {
    FindReplace,
    Sequential,
    CaseChange,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CaseMode {
    NoChange,
    Lowercase,
    Uppercase,
    TitleCase,
}

impl Default for BatchRenameState {
    fn default() -> Self {
        Self {
            open: false,
            entries: Vec::new(),
            find_text: String::new(),
            replace_text: String::new(),
            mode: RenameMode::FindReplace,
            start_number: 1,
            prefix: String::new(),
            suffix: String::new(),
            case_mode: CaseMode::NoChange,
        }
    }
}

impl BatchRenameState {
    pub fn open_with(&mut self, paths: Vec<(String, String)>) {
        self.open = true;
        self.entries = paths
            .into_iter()
            .map(|(path, name)| RenameEntry {
                original_path: path,
                original_name: name.clone(),
                new_name: name,
            })
            .collect();
        self.find_text.clear();
        self.replace_text.clear();
        self.start_number = 1;
        self.prefix.clear();
        self.suffix.clear();
        self.case_mode = CaseMode::NoChange;
        self.recompute_names();
    }

    pub fn recompute_names(&mut self) {
        match self.mode {
            RenameMode::FindReplace => {
                for entry in &mut self.entries {
                    if self.find_text.is_empty() {
                        entry.new_name = entry.original_name.clone();
                    } else {
                        entry.new_name = entry
                            .original_name
                            .replace(&self.find_text, &self.replace_text);
                    }
                }
            }
            RenameMode::Sequential => {
                for (i, entry) in self.entries.iter_mut().enumerate() {
                    let ext = Path::new(&entry.original_name)
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default();
                    let num = self.start_number + i as u32;
                    entry.new_name = format!("{}{:03}{}{}", self.prefix, num, self.suffix, ext);
                }
            }
            RenameMode::CaseChange => {
                for entry in &mut self.entries {
                    entry.new_name = match self.case_mode {
                        CaseMode::NoChange => entry.original_name.clone(),
                        CaseMode::Lowercase => entry.original_name.to_lowercase(),
                        CaseMode::Uppercase => entry.original_name.to_uppercase(),
                        CaseMode::TitleCase => to_title_case(&entry.original_name),
                    };
                }
            }
        }
    }
}

pub struct BatchRenameResult {
    pub renames: Vec<(String, String)>,
}

pub fn show(ctx: &egui::Context, state: &mut BatchRenameState) -> Option<BatchRenameResult> {
    if !state.open {
        return None;
    }

    let mut result: Option<BatchRenameResult> = None;
    let mut close = false;

    let screen = ctx.screen_rect();
    let panel_width = 650.0_f32.min(screen.width() - 40.0);

    egui::Area::new(egui::Id::new("batch_rename_backdrop"))
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let resp = ui.allocate_rect(screen, egui::Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(140));
            if resp.clicked() {
                close = true;
            }
        });

    egui::Window::new("Batch Rename")
        .collapsible(false)
        .resizable(false)
        .default_width(panel_width)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(panel_width - 40.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} files selected", state.entries.len()))
                        .color(theme::SECONDARY)
                        .size(12.0),
                );
            });

            ui.add_space(8.0);

            let mut mode_changed = false;
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        state.mode == RenameMode::FindReplace,
                        egui::RichText::new("Find & Replace").size(12.0),
                    )
                    .clicked()
                {
                    state.mode = RenameMode::FindReplace;
                    mode_changed = true;
                }
                if ui
                    .selectable_label(
                        state.mode == RenameMode::Sequential,
                        egui::RichText::new("Sequential").size(12.0),
                    )
                    .clicked()
                {
                    state.mode = RenameMode::Sequential;
                    mode_changed = true;
                }
                if ui
                    .selectable_label(
                        state.mode == RenameMode::CaseChange,
                        egui::RichText::new("Case").size(12.0),
                    )
                    .clicked()
                {
                    state.mode = RenameMode::CaseChange;
                    mode_changed = true;
                }
            });

            ui.add_space(8.0);

            let mut params_changed = mode_changed;

            match state.mode {
                RenameMode::FindReplace => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Find:")
                                .color(theme::SECONDARY)
                                .size(12.0),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut state.find_text)
                                    .desired_width(200.0),
                            )
                            .changed()
                        {
                            params_changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Replace:")
                                .color(theme::SECONDARY)
                                .size(12.0),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut state.replace_text)
                                    .desired_width(200.0),
                            )
                            .changed()
                        {
                            params_changed = true;
                        }
                    });
                }
                RenameMode::Sequential => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Prefix:")
                                .color(theme::SECONDARY)
                                .size(12.0),
                        );
                        if ui
                            .add(egui::TextEdit::singleline(&mut state.prefix).desired_width(120.0))
                            .changed()
                        {
                            params_changed = true;
                        }
                        ui.label(
                            egui::RichText::new("Start #:")
                                .color(theme::SECONDARY)
                                .size(12.0),
                        );
                        if ui
                            .add(egui::DragValue::new(&mut state.start_number).range(0..=9999))
                            .changed()
                        {
                            params_changed = true;
                        }
                        ui.label(
                            egui::RichText::new("Suffix:")
                                .color(theme::SECONDARY)
                                .size(12.0),
                        );
                        if ui
                            .add(egui::TextEdit::singleline(&mut state.suffix).desired_width(120.0))
                            .changed()
                        {
                            params_changed = true;
                        }
                    });
                }
                RenameMode::CaseChange => {
                    ui.horizontal(|ui| {
                        for (label, mode) in [
                            ("lowercase", CaseMode::Lowercase),
                            ("UPPERCASE", CaseMode::Uppercase),
                            ("Title Case", CaseMode::TitleCase),
                        ] {
                            if ui
                                .selectable_label(
                                    state.case_mode == mode,
                                    egui::RichText::new(label).size(12.0),
                                )
                                .clicked()
                            {
                                state.case_mode = mode;
                                params_changed = true;
                            }
                        }
                    });
                }
            }

            if params_changed {
                state.recompute_names();
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Preview")
                    .color(theme::TEXT)
                    .size(12.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let changed_count = state
                .entries
                .iter()
                .filter(|e| e.new_name != e.original_name)
                .count();

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("batch_rename_preview")
                        .num_columns(3)
                        .spacing([8.0, 3.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Original")
                                    .color(theme::SECONDARY)
                                    .size(11.0)
                                    .strong(),
                            );
                            ui.label(egui::RichText::new("→").color(theme::MUTED).size(11.0));
                            ui.label(
                                egui::RichText::new("New Name")
                                    .color(theme::SECONDARY)
                                    .size(11.0)
                                    .strong(),
                            );
                            ui.end_row();

                            for entry in &state.entries {
                                let changed = entry.new_name != entry.original_name;
                                let name_color = if changed { theme::TEXT } else { theme::MUTED };

                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&entry.original_name)
                                            .color(theme::SECONDARY)
                                            .size(11.0),
                                    )
                                    .truncate(),
                                );
                                ui.label(egui::RichText::new("→").color(theme::MUTED).size(11.0));
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&entry.new_name)
                                            .color(name_color)
                                            .size(11.0),
                                    )
                                    .truncate(),
                                );
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} will be renamed", changed_count))
                        .color(theme::SECONDARY)
                        .size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_apply = changed_count > 0;
                    if ui
                        .add_enabled(can_apply, egui::Button::new("Rename"))
                        .clicked()
                    {
                        let renames: Vec<(String, String)> = state
                            .entries
                            .iter()
                            .filter(|e| e.new_name != e.original_name)
                            .map(|e| {
                                let new_path = Path::new(&e.original_path)
                                    .with_file_name(&e.new_name)
                                    .to_string_lossy()
                                    .to_string();
                                (e.original_path.clone(), new_path)
                            })
                            .collect();
                        result = Some(BatchRenameResult { renames });
                        close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });
        });

    if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.open = false;
    }

    result
}

fn to_title_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == ' ' || c == '_' || c == '-' || c == '.' {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }
    result
}
