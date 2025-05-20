// src/ui.rs

use egui;
// Assuming ui.rs is a local module of main.rs (`mod ui;` in main.rs)
// It needs to access ConvexPolygon from the library
use convex_polygon_intersection::geometry::ConvexPolygon;

// ... rest of build_ui function (no changes needed to logic)
pub fn build_ui(
    ctx: &egui::Context,
    polygon1: &ConvexPolygon,
    polygon2: &ConvexPolygon,
    intersection: &ConvexPolygon,
    is_animating: &mut bool,
    regenerate_requested: &mut bool,
) {
    egui::Window::new("Polygon Controls")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                if ui.button("🔄 Generate New Polygons").clicked() {
                    *regenerate_requested = true;
                }

                ui.separator();

                ui.label("📊 Polygon Statistics:");
                ui.label(format!("🔴 Polygon 1: {} vertices, Area: {:.1}",
                    polygon1.count(), polygon1.area()));
                ui.label(format!("🔵 Polygon 2: {} vertices, Area: {:.1}",
                    polygon2.count(), polygon2.area()));
                ui.label(format!("🟢 Intersection: {} vertices, Area: {:.1}",
                    intersection.count(), intersection.area()));

                ui.separator();

                ui.checkbox(is_animating, "🎬 Enable Animation");

                ui.separator();

                ui.label("🎮 Keyboard Controls:");
                ui.label("   Space: Generate new polygons");
                ui.label("   S: Print stats to console");
                ui.label("   A: Toggle animation");
                ui.label("   T: Run performance benchmark"); // Added T to controls list
            });
        });
}