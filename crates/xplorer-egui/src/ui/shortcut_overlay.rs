use eframe::egui;

use crate::theme;

pub struct ShortcutOverlayState {
    pub open: bool,
}

impl Default for ShortcutOverlayState {
    fn default() -> Self {
        Self { open: false }
    }
}

struct ShortcutGroup {
    title: &'static str,
    items: &'static [(&'static str, &'static str)],
}

const GROUPS: &[ShortcutGroup] = &[
    ShortcutGroup {
        title: "Navigation",
        items: &[
            ("Alt+←", "Go back"),
            ("Alt+→", "Go forward"),
            ("Alt+↑", "Go to parent"),
            ("Backspace", "Go to parent"),
            ("Enter", "Open / Enter directory"),
            ("Ctrl+L", "Edit address bar"),
        ],
    },
    ShortcutGroup {
        title: "Tabs & Panels",
        items: &[
            ("Ctrl+T", "New tab"),
            ("Ctrl+W", "Close tab"),
            ("Ctrl+Tab", "Next tab"),
            ("Ctrl+Shift+Tab", "Previous tab"),
            ("Tab", "Switch pane focus"),
            ("Ctrl+B", "Toggle sidebar"),
            ("Ctrl+P", "Toggle preview panel"),
        ],
    },
    ShortcutGroup {
        title: "File Operations",
        items: &[
            ("Ctrl+C", "Copy"),
            ("Ctrl+X", "Cut"),
            ("Ctrl+V", "Paste"),
            ("F2", "Rename"),
            ("Del", "Move to Trash"),
            ("Shift+Del", "Delete permanently"),
            ("Alt+Enter", "Properties"),
        ],
    },
    ShortcutGroup {
        title: "Selection",
        items: &[
            ("↑ / ↓", "Move selection"),
            ("Ctrl+A", "Select all"),
            ("Ctrl+Click", "Toggle item"),
            ("Shift+Click", "Range select"),
        ],
    },
    ShortcutGroup {
        title: "View & Search",
        items: &[
            ("Ctrl+F", "Filter current listing"),
            ("Ctrl+Shift+F", "Deep search"),
            ("Ctrl+H", "Toggle hidden files"),
            ("Ctrl+K", "Command palette"),
            ("Ctrl+1", "Details view"),
            ("Ctrl+2", "Grid view"),
            ("F5", "Refresh"),
            ("?", "This overlay"),
        ],
    },
];

pub fn show(ctx: &egui::Context, state: &mut ShortcutOverlayState) {
    if !state.open {
        return;
    }

    let screen_rect = ctx.screen_rect();

    egui::Area::new(egui::Id::new("shortcut_overlay_backdrop"))
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let resp = ui.allocate_rect(screen_rect, egui::Sense::click());
            ui.painter()
                .rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(140));
            if resp.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.open = false;
            }
        });

    let overlay_width = 560.0_f32.min(screen_rect.width() - 40.0);
    let overlay_x = (screen_rect.width() - overlay_width) / 2.0;
    let overlay_y = screen_rect.height() * 0.1;

    egui::Area::new(egui::Id::new("shortcut_overlay"))
        .fixed_pos(egui::pos2(overlay_x, overlay_y))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::SURFACE)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, theme::BORDER))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_width(overlay_width);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Keyboard Shortcuts")
                                .color(theme::TEXT)
                                .size(16.0)
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Press Esc to close")
                                    .color(theme::MUTED)
                                    .size(11.0),
                            );
                        });
                    });

                    ui.add_space(12.0);

                    let col_width = (overlay_width - 24.0) / 2.0;

                    ui.columns(2, |cols| {
                        let left_groups = &GROUPS[..3];
                        let right_groups = &GROUPS[3..];

                        for group in left_groups {
                            render_group(&mut cols[0], group, col_width);
                        }
                        for group in right_groups {
                            render_group(&mut cols[1], group, col_width);
                        }
                    });
                });
        });
}

fn render_group(ui: &mut egui::Ui, group: &ShortcutGroup, _width: f32) {
    ui.label(
        egui::RichText::new(group.title)
            .color(theme::SELECTION)
            .size(12.0)
            .strong(),
    );
    ui.add_space(4.0);

    for &(key, desc) in group.items {
        ui.horizontal(|ui| {
            ui.set_min_width(80.0);
            egui::Frame::new()
                .fill(theme::BACKGROUND)
                .corner_radius(3.0)
                .inner_margin(egui::Margin {
                    left: 4,
                    right: 4,
                    top: 1,
                    bottom: 1,
                })
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(key)
                            .color(theme::TEXT)
                            .size(11.0)
                            .family(egui::FontFamily::Monospace),
                    );
                });
            ui.label(egui::RichText::new(desc).color(theme::SECONDARY).size(11.0));
        });
    }

    ui.add_space(10.0);
}
