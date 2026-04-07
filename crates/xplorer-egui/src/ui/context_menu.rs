use eframe::egui;

use crate::theme;

pub enum FileContextAction {
    None,
    Open,
    OpenInTerminal,
    ShowInFolder,
    CopyPath,
    Copy,
    Cut,
    Paste,
    Delete,
    MoveToTrash,
    Rename,
    AddToFavorites,
}

pub enum EmptyAreaAction {
    None,
    NewFolder,
    NewFile,
    Paste,
    Refresh,
    OpenInTerminal,
}

fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: Option<&str>) -> bool {
    let mut btn = egui::Button::new(label).frame(false);
    if let Some(sc) = shortcut {
        btn = btn.shortcut_text(egui::RichText::new(sc).color(theme::MUTED).size(12.0));
    }
    ui.add_sized([ui.available_width().max(180.0), 0.0], btn)
        .clicked()
}

pub fn file_context_menu(
    response: &egui::Response,
    is_dir: bool,
    has_clipboard: bool,
) -> FileContextAction {
    let mut action = FileContextAction::None;

    response.context_menu(|ui| {
        if menu_item(ui, "Open", None) {
            action = FileContextAction::Open;
            ui.close_menu();
        }
        if is_dir && menu_item(ui, "Open in Terminal", None) {
            action = FileContextAction::OpenInTerminal;
            ui.close_menu();
        }
        ui.separator();
        if menu_item(ui, "Copy", Some("Ctrl+C")) {
            action = FileContextAction::Copy;
            ui.close_menu();
        }
        if menu_item(ui, "Cut", Some("Ctrl+X")) {
            action = FileContextAction::Cut;
            ui.close_menu();
        }
        if has_clipboard && menu_item(ui, "Paste", Some("Ctrl+V")) {
            action = FileContextAction::Paste;
            ui.close_menu();
        }
        ui.separator();
        if menu_item(ui, "Rename", Some("F2")) {
            action = FileContextAction::Rename;
            ui.close_menu();
        }
        if menu_item(ui, "Move to Trash", Some("Del")) {
            action = FileContextAction::MoveToTrash;
            ui.close_menu();
        }
        if menu_item(ui, "Delete permanently", Some("Shift+Del")) {
            action = FileContextAction::Delete;
            ui.close_menu();
        }
        ui.separator();
        if menu_item(ui, "Copy path", None) {
            action = FileContextAction::CopyPath;
            ui.close_menu();
        }
        if menu_item(ui, "Show in Explorer", None) {
            action = FileContextAction::ShowInFolder;
            ui.close_menu();
        }
        if is_dir && menu_item(ui, "Add to Favorites", None) {
            action = FileContextAction::AddToFavorites;
            ui.close_menu();
        }
    });

    action
}

pub fn empty_area_context_menu(response: &egui::Response, has_clipboard: bool) -> EmptyAreaAction {
    let mut action = EmptyAreaAction::None;

    response.context_menu(|ui| {
        if menu_item(ui, "New Folder", None) {
            action = EmptyAreaAction::NewFolder;
            ui.close_menu();
        }
        if menu_item(ui, "New File", None) {
            action = EmptyAreaAction::NewFile;
            ui.close_menu();
        }
        ui.separator();
        if has_clipboard && menu_item(ui, "Paste", Some("Ctrl+V")) {
            action = EmptyAreaAction::Paste;
            ui.close_menu();
        }
        if menu_item(ui, "Refresh", Some("F5")) {
            action = EmptyAreaAction::Refresh;
            ui.close_menu();
        }
        if menu_item(ui, "Open in Terminal", None) {
            action = EmptyAreaAction::OpenInTerminal;
            ui.close_menu();
        }
    });

    action
}
