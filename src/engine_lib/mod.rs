// src/engine_lib/mod.rs

pub mod scene_types;
pub mod camera;
pub mod controller;
pub mod side_handler; // Added new module

// Re-export key types from the new structure
pub use scene_types::{
    Point3, Mat4, Scene, HullBlueprint, HullInstance, BlueprintSide,
    HandlerConfig, SideHandlerTypeId, PortalConnectionInfo, TraversalState,
    InstanceId, BlueprintId, PortalId, SideIndex,
};
pub use camera::Camera;
pub use controller::CameraController;
pub use side_handler::{SideHandler, StandardWallHandler, StandardPortalHandler, HandlerContext, MAX_PORTAL_RECURSION_DEPTH};