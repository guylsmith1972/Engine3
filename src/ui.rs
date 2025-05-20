// src/ui.rs
use egui;

pub fn build_ui(ctx: &egui::Context) { // Removed ConvexPolygon and control bools
    egui::Window::new("Controls & Info") // Renamed window slightly
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Portal Rendering Demo");
                ui.separator();

                // Add any relevant 3D app status/info here if needed in the future.
                // For now, it will be minimal.

                ui.label("🎮 Keyboard Controls:");
                ui.label("   W/A/S/D: Move Camera");
                ui.label("   Space: Move Up");
                ui.label("   L-Shift/L-Ctrl: Move Down");
                ui.label("   Arrow Keys: Look Up/Down/Left/Right");
                ui.label("   Mouse (when grabbed): Look");
                ui.label("   Escape: Grab/Ungrab Mouse Cursor");
                // "T: Run performance benchmark" can be kept if you still want users to know.
                // The benchmark itself (intersection_benchmark.rs) is separate from the app's runtime.
                // ui.label("   T: Run performance benchmark (via 'cargo bench')");
            });
        });
}