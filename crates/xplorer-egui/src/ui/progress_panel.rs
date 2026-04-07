use eframe::egui;

use crate::state::OperationProgress;
use crate::theme;

pub fn show(ctx: &egui::Context, operations: &[OperationProgress]) {
    if operations.is_empty() {
        return;
    }

    let screen = ctx.screen_rect();
    let panel_width = 320.0_f32;
    let x = screen.max.x - panel_width - 16.0;
    let y = screen.max.y - 16.0 - (operations.len() as f32 * 64.0);

    egui::Area::new(egui::Id::new("op_progress_panel"))
        .fixed_pos(egui::pos2(x, y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            for op in operations {
                egui::Frame::new()
                    .fill(theme::SURFACE)
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(panel_width);

                        let fraction = if op.total > 0 {
                            op.completed as f32 / op.total as f32
                        } else {
                            0.0
                        };

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&op.op_label)
                                    .color(theme::TEXT)
                                    .size(12.0)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}/{}",
                                            op.completed, op.total
                                        ))
                                        .color(theme::SECONDARY)
                                        .size(11.0),
                                    );
                                },
                            );
                        });

                        ui.add_space(4.0);

                        let bar =
                            egui::ProgressBar::new(fraction).desired_width(panel_width - 20.0);
                        ui.add(bar);

                        if !op.current_name.is_empty() {
                            ui.label(
                                egui::RichText::new(&op.current_name)
                                    .color(theme::MUTED)
                                    .size(10.0),
                            );
                        }
                    });

                ui.add_space(4.0);
            }
        });
}
