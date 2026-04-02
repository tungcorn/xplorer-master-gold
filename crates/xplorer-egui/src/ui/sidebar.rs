use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub enum SidebarAction {
    None,
    Navigate(String),
    RemoveBookmark(String),
}

pub fn show(ctx: &egui::Context, state: &mut AppState) -> SidebarAction {
    let mut action = SidebarAction::None;

    if !state.show_sidebar {
        return action;
    }

    egui::SidePanel::left("sidebar")
        .default_width(220.0)
        .width_range(150.0..=400.0)
        .resizable(true)
        .frame(egui::Frame::side_top_panel(&ctx.style()).fill(theme::CAPTION))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            show_quick_access(ui, &mut action);
            ui.add_space(12.0);
            show_drives_section(ui, state, &mut action);
            ui.add_space(12.0);
            show_favorites_section(ui, state, &mut action);
        });

    action
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label)
            .color(theme::MUTED)
            .size(11.0)
            .strong(),
    );
    ui.add_space(4.0);
}

fn show_quick_access(ui: &mut egui::Ui, action: &mut SidebarAction) {
    section_header(ui, "Quick Access");

    let quick_items = [
        ("Home", dirs::home_dir()),
        ("Desktop", dirs::desktop_dir()),
        ("Documents", dirs::document_dir()),
        ("Downloads", dirs::download_dir()),
    ];

    for (label, path_opt) in &quick_items {
        if let Some(path) = path_opt {
            let path_str = path.to_string_lossy().to_string();
            let text = egui::RichText::new(*label).color(theme::TEXT).size(13.0);
            if ui.selectable_label(false, text).clicked() {
                *action = SidebarAction::Navigate(path_str);
            }
        }
    }
}

fn show_drives_section(ui: &mut egui::Ui, state: &AppState, action: &mut SidebarAction) {
    section_header(ui, "Drives");

    for drive in &state.drives {
        let label = format!("{} ({})", drive.name, drive.mount_point);
        let text = egui::RichText::new(&label).color(theme::TEXT).size(13.0);
        let response = ui.selectable_label(false, text);

        let used = drive.total_space.saturating_sub(drive.available_space);
        let fraction = if drive.total_space > 0 {
            used as f32 / drive.total_space as f32
        } else {
            0.0
        };
        let bar_color = if fraction > 0.9 {
            theme::WARNING
        } else {
            theme::SELECTION
        };
        ui.add(
            egui::ProgressBar::new(fraction)
                .desired_width(ui.available_width() - 8.0)
                .fill(bar_color),
        );
        ui.add_space(2.0);

        if response.clicked() {
            *action = SidebarAction::Navigate(drive.mount_point.clone());
        }
    }
}

fn show_favorites_section(ui: &mut egui::Ui, state: &AppState, action: &mut SidebarAction) {
    section_header(ui, "Favorites");

    if state.bookmarks.is_empty() {
        ui.label(
            egui::RichText::new("No favorites yet")
                .color(theme::MUTED)
                .size(12.0)
                .italics(),
        );
        return;
    }

    for bookmark in &state.bookmarks {
        let text = egui::RichText::new(&bookmark.name)
            .color(theme::TEXT)
            .size(13.0);
        let response = ui.selectable_label(false, text);
        if response.clicked() {
            *action = SidebarAction::Navigate(bookmark.path.clone());
        }
        response.context_menu(|ui| {
            if ui.button("Remove from favorites").clicked() {
                *action = SidebarAction::RemoveBookmark(bookmark.path.clone());
                ui.close_menu();
            }
        });
    }
}
