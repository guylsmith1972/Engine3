// engine_lib/src/lib.rs
pub mod scene_types; // Renamed from scene.rs to avoid confusion, contains only types
pub mod camera;
pub mod controller; // New file for input handling

pub use scene_types::{Point3, SceneSide, Hull, Scene, TraversalState};
pub use camera::Camera;
pub use controller::CameraController; // Or whatever the main struct in controller.rs is