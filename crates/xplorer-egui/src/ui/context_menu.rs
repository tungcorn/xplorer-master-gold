use eframe::egui;

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

pub fn file_context_menu(
    response: &egui::Response,
    is_dir: bool,
    has_clipboard: bool,
) -> FileContextAction {
    let mut action = FileContextAction::None;

    response.context_menu(|ui| {
        if ui.button("Open").clicked() {
            action = FileContextAction::Open;
            ui.close_menu();
        }
        if is_dir && ui.button("Open in Terminal").clicked() {
            action = FileContextAction::OpenInTerminal;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Copy               Ctrl+C").clicked() {
            action = FileContextAction::Copy;
            ui.close_menu();
        }
        if ui.button("Cut                Ctrl+X").clicked() {
            action = FileContextAction::Cut;
            ui.close_menu();
        }
        if has_clipboard && ui.button("Paste              Ctrl+V").clicked() {
            action = FileContextAction::Paste;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Rename             F2").clicked() {
            action = FileContextAction::Rename;
            ui.close_menu();
        }
        if ui.button("Move to Trash      Del").clicked() {
            action = FileContextAction::MoveToTrash;
            ui.close_menu();
        }
        if ui.button("Delete permanently Shift+Del").clicked() {
            action = FileContextAction::Delete;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Copy path").clicked() {
            action = FileContextAction::CopyPath;
            ui.close_menu();
        }
        if ui.button("Show in Explorer").clicked() {
            action = FileContextAction::ShowInFolder;
            ui.close_menu();
        }
        if is_dir && ui.button("Add to Favorites").clicked() {
            action = FileContextAction::AddToFavorites;
            ui.close_menu();
        }
    });

    action
}

pub fn empty_area_context_menu(response: &egui::Response, has_clipboard: bool) -> EmptyAreaAction {
    let mut action = EmptyAreaAction::None;

    response.context_menu(|ui| {
        if ui.button("New Folder").clicked() {
            action = EmptyAreaAction::NewFolder;
            ui.close_menu();
        }
        if ui.button("New File").clicked() {
            action = EmptyAreaAction::NewFile;
            ui.close_menu();
        }
        ui.separator();
        if has_clipboard && ui.button("Paste              Ctrl+V").clicked() {
            action = EmptyAreaAction::Paste;
            ui.close_menu();
        }
        if ui.button("Refresh").clicked() {
            action = EmptyAreaAction::Refresh;
            ui.close_menu();
        }
        if ui.button("Open in Terminal").clicked() {
            action = EmptyAreaAction::OpenInTerminal;
            ui.close_menu();
        }
    });

    action
}
