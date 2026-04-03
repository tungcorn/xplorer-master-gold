use eframe::egui;

use crate::icons;
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
        .default_width(228.0)
        .width_range(160.0..=400.0)
        .resizable(true)
        .frame(
            egui::Frame::NONE
                .fill(theme::CAPTION)
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(ctx, |ui| {
            ui.add_space(4.0);
            show_quick_access(ui, state, &mut action);
            ui.add_space(8.0);
            show_drives_section(ui, state, &mut action);
            ui.add_space(8.0);
            show_favorites_section(ui, state, &mut action);
        });

    action
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label.to_uppercase())
            .color(theme::MUTED)
            .size(11.0)
            .strong(),
    );
    let rect = ui.available_rect_before_wrap();
    let line_y = rect.top() + 2.0;
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), line_y),
            egui::pos2(rect.right(), line_y),
        ],
        egui::Stroke::new(1.0, theme::BORDER),
    );
    ui.add_space(6.0);
}

fn sidebar_item(ui: &mut egui::Ui, icon: &str, label: &str, is_active: bool) -> egui::Response {
    let text_color = if is_active {
        theme::TEXT
    } else {
        theme::SECONDARY
    };
    let text = format!("{}  {}", icon, label);
    let rich = egui::RichText::new(text).color(text_color).size(14.0);
    ui.selectable_label(is_active, rich)
}

fn show_quick_access(ui: &mut egui::Ui, state: &AppState, action: &mut SidebarAction) {
    section_header(ui, "Quick Access");

    let active_path = &state.active_tab().path;
    let quick_items: [(&str, &str, Option<std::path::PathBuf>); 4] = [
        (icons::HOME, "Home", dirs::home_dir()),
        (icons::DESKTOP, "Desktop", dirs::desktop_dir()),
        (icons::DOCUMENTS, "Documents", dirs::document_dir()),
        (icons::DOWNLOADS, "Downloads", dirs::download_dir()),
    ];

    for (icon, label, path_opt) in &quick_items {
        if let Some(path) = path_opt {
            let path_str = path.to_string_lossy().to_string();
            let is_active = *active_path == path_str;
            if sidebar_item(ui, icon, label, is_active).clicked() {
                *action = SidebarAction::Navigate(path_str);
            }
        }
    }
}

fn show_drives_section(ui: &mut egui::Ui, state: &AppState, action: &mut SidebarAction) {
    section_header(ui, "Drives");

    let active_path = &state.active_tab().path;

    for drive in &state.drives {
        let label = format!("{} ({})", drive.name, drive.mount_point);
        let is_active = active_path.starts_with(&drive.mount_point)
            && state
                .drives
                .iter()
                .filter(|d| active_path.starts_with(&d.mount_point))
                .all(|d| d.mount_point.len() <= drive.mount_point.len());

        if sidebar_item(ui, icons::DRIVE, &label, is_active).clicked() {
            *action = SidebarAction::Navigate(drive.mount_point.clone());
        }

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

        let bar_width = ui.available_width();
        let bar_height = 4.0;
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
        ui.painter()
            .rect_filled(bar_rect, egui::CornerRadius::same(2), theme::SURFACE);
        let fill_width = bar_width * fraction;
        if fill_width > 0.0 {
            let fill_rect =
                egui::Rect::from_min_size(bar_rect.min, egui::vec2(fill_width, bar_height));
            ui.painter()
                .rect_filled(fill_rect, egui::CornerRadius::same(2), bar_color);
        }

        let capacity_text = format!(
            "{} / {}",
            format_drive_size(used),
            format_drive_size(drive.total_space)
        );
        ui.label(
            egui::RichText::new(capacity_text)
                .color(theme::MUTED)
                .size(11.0),
        );
        ui.add_space(4.0);
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

    let active_path = &state.active_tab().path;

    for bookmark in &state.bookmarks {
        let is_active = *active_path == bookmark.path;
        let response = sidebar_item(ui, icons::BOOKMARK, &bookmark.name, is_active);
        if response.clicked() {
            *action = SidebarAction::Navigate(bookmark.path.clone());
        }
        response.context_menu(|ui| {
            if ui
                .button(egui::RichText::new("Remove from favorites").color(theme::WARNING))
                .clicked()
            {
                *action = SidebarAction::RemoveBookmark(bookmark.path.clone());
                ui.close_menu();
            }
        });
    }
}

fn format_drive_size(bytes: u64) -> String {
    if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
