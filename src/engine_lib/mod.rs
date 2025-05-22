// src/engine_lib/mod.rs

pub mod scene_types;
pub mod camera;
pub mod controller;
pub mod side_handler;

// Re-export key types from the new structure
// Point3 and Mat4 are no longer custom types from scene_types,
// so they are not re-exported here. Consumers should use glam::Vec3 and glam::Mat4.
pub use scene_types::{
    Scene, HullBlueprint, HullInstance, BlueprintSide,
    HandlerConfig, SideHandlerTypeId, PortalConnectionInfo, TraversalState,
    InstanceId, BlueprintId, PortalId, SideIndex,
};
pub use camera::Camera;
pub use controller::CameraController;
pub use side_handler::{SideHandler, StandardWallHandler, StandardPortalHandler, HandlerContext, MAX_PORTAL_RECURSION_DEPTH};