use eframe::egui;

use crate::theme;

pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected_index: usize,
    focus_requested: bool,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected_index: 0,
            focus_requested: false,
        }
    }
}

impl CommandPaletteState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.selected_index = 0;
            self.focus_requested = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
    }
}

#[derive(Clone)]
pub struct PaletteEntry {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: PaletteAction,
}

#[derive(Clone)]
pub enum PaletteAction {
    ToggleSidebar,
    TogglePreview,
    ToggleHiddenFiles,
    NewTab,
    CloseTab,
    GoBack,
    GoForward,
    GoUp,
    GoHome,
    Refresh,
    FocusFilter,
    EditAddressBar,
    SplitRight,
    CopySelection,
    CutSelection,
    PasteClipboard,
    Rename,
    MoveToTrash,
    DeletePermanently,
    SelectAll,
    BatchRename,
    #[allow(dead_code)]
    NavigateTo(String),
}

fn all_commands() -> Vec<PaletteEntry> {
    vec![
        PaletteEntry {
            label: "Toggle Sidebar".into(),
            shortcut: Some("Ctrl+B".into()),
            action: PaletteAction::ToggleSidebar,
        },
        PaletteEntry {
            label: "Toggle Preview Panel".into(),
            shortcut: Some("Ctrl+P".into()),
            action: PaletteAction::TogglePreview,
        },
        PaletteEntry {
            label: "Toggle Hidden Files".into(),
            shortcut: Some("Ctrl+H".into()),
            action: PaletteAction::ToggleHiddenFiles,
        },
        PaletteEntry {
            label: "New Tab".into(),
            shortcut: Some("Ctrl+T".into()),
            action: PaletteAction::NewTab,
        },
        PaletteEntry {
            label: "Close Tab".into(),
            shortcut: Some("Ctrl+W".into()),
            action: PaletteAction::CloseTab,
        },
        PaletteEntry {
            label: "Go Back".into(),
            shortcut: Some("Alt+←".into()),
            action: PaletteAction::GoBack,
        },
        PaletteEntry {
            label: "Go Forward".into(),
            shortcut: Some("Alt+→".into()),
            action: PaletteAction::GoForward,
        },
        PaletteEntry {
            label: "Go Up".into(),
            shortcut: Some("Alt+↑".into()),
            action: PaletteAction::GoUp,
        },
        PaletteEntry {
            label: "Go Home".into(),
            shortcut: None,
            action: PaletteAction::GoHome,
        },
        PaletteEntry {
            label: "Refresh".into(),
            shortcut: Some("F5".into()),
            action: PaletteAction::Refresh,
        },
        PaletteEntry {
            label: "Filter Files".into(),
            shortcut: Some("Ctrl+F".into()),
            action: PaletteAction::FocusFilter,
        },
        PaletteEntry {
            label: "Edit Address Bar".into(),
            shortcut: Some("Ctrl+L".into()),
            action: PaletteAction::EditAddressBar,
        },
        PaletteEntry {
            label: "Split Right".into(),
            shortcut: None,
            action: PaletteAction::SplitRight,
        },
        PaletteEntry {
            label: "Copy".into(),
            shortcut: Some("Ctrl+C".into()),
            action: PaletteAction::CopySelection,
        },
        PaletteEntry {
            label: "Cut".into(),
            shortcut: Some("Ctrl+X".into()),
            action: PaletteAction::CutSelection,
        },
        PaletteEntry {
            label: "Paste".into(),
            shortcut: Some("Ctrl+V".into()),
            action: PaletteAction::PasteClipboard,
        },
        PaletteEntry {
            label: "Rename".into(),
            shortcut: Some("F2".into()),
            action: PaletteAction::Rename,
        },
        PaletteEntry {
            label: "Move to Trash".into(),
            shortcut: Some("Del".into()),
            action: PaletteAction::MoveToTrash,
        },
        PaletteEntry {
            label: "Delete Permanently".into(),
            shortcut: Some("Shift+Del".into()),
            action: PaletteAction::DeletePermanently,
        },
        PaletteEntry {
            label: "Select All".into(),
            shortcut: Some("Ctrl+A".into()),
            action: PaletteAction::SelectAll,
        },
        PaletteEntry {
            label: "Batch Rename...".into(),
            shortcut: Some("Ctrl+Shift+R".into()),
            action: PaletteAction::BatchRename,
        },
    ]
}

fn fuzzy_match(query: &str, label: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_lowercase();
    let label_lower = label.to_lowercase();
    let mut label_chars = label_lower.chars();
    for qc in query_lower.chars() {
        if label_chars.find(|&lc| lc == qc).is_none() {
            return false;
        }
    }
    true
}

pub fn show(ctx: &egui::Context, palette: &mut CommandPaletteState) -> Option<PaletteAction> {
    if !palette.open {
        return None;
    }

    let mut action_result: Option<PaletteAction> = None;

    let screen_rect = ctx.screen_rect();
    let overlay_width = 500.0_f32.min(screen_rect.width() - 40.0);
    let overlay_x = (screen_rect.width() - overlay_width) / 2.0;
    let overlay_y = screen_rect.height() * 0.2;

    egui::Area::new(egui::Id::new("command_palette_backdrop"))
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let backdrop = ui.allocate_rect(screen_rect, egui::Sense::click());
            ui.painter()
                .rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(120));
            if backdrop.clicked() {
                palette.close();
            }
        });

    let commands = all_commands();
    let filtered: Vec<&PaletteEntry> = commands
        .iter()
        .filter(|e| fuzzy_match(&palette.query, &e.label))
        .collect();

    if palette.selected_index >= filtered.len() && !filtered.is_empty() {
        palette.selected_index = filtered.len() - 1;
    }

    egui::Area::new(egui::Id::new("command_palette"))
        .fixed_pos(egui::pos2(overlay_x, overlay_y))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::SURFACE)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, theme::BORDER))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(80),
                })
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_width(overlay_width);

                    let input_resp = ui.add_sized(
                        [overlay_width - 16.0, 28.0],
                        egui::TextEdit::singleline(&mut palette.query)
                            .hint_text("Type a command…")
                            .frame(false)
                            .font(egui::TextStyle::Body),
                    );

                    if palette.focus_requested {
                        input_resp.request_focus();
                        palette.focus_requested = false;
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        palette.close();
                        return;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        if !filtered.is_empty() {
                            palette.selected_index =
                                (palette.selected_index + 1).min(filtered.len() - 1);
                        }
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        palette.selected_index = palette.selected_index.saturating_sub(1);
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !filtered.is_empty() {
                        action_result = Some(filtered[palette.selected_index].action.clone());
                        palette.close();
                        return;
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let max_visible = 10;
                    egui::ScrollArea::vertical()
                        .max_height(max_visible as f32 * 32.0)
                        .show(ui, |ui| {
                            for (i, entry) in filtered.iter().enumerate() {
                                let is_active = i == palette.selected_index;
                                let bg = if is_active {
                                    theme::HOVER
                                } else {
                                    egui::Color32::TRANSPARENT
                                };

                                let resp = egui::Frame::new()
                                    .fill(bg)
                                    .corner_radius(4.0)
                                    .inner_margin(egui::Margin {
                                        left: 8,
                                        right: 8,
                                        top: 4,
                                        bottom: 4,
                                    })
                                    .show(ui, |ui| {
                                        ui.set_width(overlay_width - 32.0);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&entry.label)
                                                    .color(if is_active {
                                                        theme::TEXT
                                                    } else {
                                                        theme::SECONDARY
                                                    })
                                                    .size(13.0),
                                            );
                                            if let Some(ref sc) = entry.shortcut {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            egui::RichText::new(sc)
                                                                .color(theme::MUTED)
                                                                .size(11.0),
                                                        );
                                                    },
                                                );
                                            }
                                        });
                                    });

                                if resp.response.clicked() {
                                    action_result = Some(entry.action.clone());
                                    palette.close();
                                    return;
                                }
                                if resp.response.hovered() && !is_active {
                                    palette.selected_index = i;
                                }
                            }

                            if filtered.is_empty() {
                                ui.label(
                                    egui::RichText::new("No matching commands")
                                        .color(theme::MUTED)
                                        .size(12.0)
                                        .italics(),
                                );
                            }
                        });
                });
        });

    action_result
}
