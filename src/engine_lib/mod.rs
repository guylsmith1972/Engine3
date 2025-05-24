// src/engine_lib/mod.rs

pub mod scene_types;
pub mod camera;
pub mod controller;
pub mod side_handler;
pub mod scene_logic; // Added new module

pub use scene_types::{
    Scene, HullBlueprint, HullInstance, BlueprintSide,
    HandlerConfig, SideHandlerTypeId, PortalConnectionInfo, TraversalState, BoundaryCheckResult,
    InstanceId, BlueprintId, PortalId, SideIndex,
};
pub use camera::Camera;
pub use controller::CameraController;
pub use side_handler::{
    SideHandler, StandardWallHandler, StandardPortalHandler, HandlerContext,
    MAX_PORTAL_RECURSION_DEPTH, get_portal_alignment_transform,
};
pub use scene_logic::{update_camera_in_scene, check_camera_hull_boundary}; // Re-export new functions