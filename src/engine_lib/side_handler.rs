// src/engine_lib/side_handler.rs

use std::collections::VecDeque;
use glam::{Mat4, Vec3};
use crate::engine_lib::scene_types::{
    Scene, HandlerConfig,
    HullInstance, BlueprintSide, TraversalState, PortalId,
};
use crate::engine_lib::camera::Camera;
use crate::rendering_lib::geometry::ConvexPolygon;
use crate::rendering_lib::vertex::Vertex;
use crate::demo_scene::{
    PORTAL_ID_FRONT, PORTAL_ID_BACK, PORTAL_ID_LEFT, PORTAL_ID_RIGHT, PORTAL_ID_TOP, PORTAL_ID_BOTTOM,
};

pub const MAX_PORTAL_RECURSION_DEPTH: u32 = 10;

pub struct HandlerContext<'a> {
    pub frame_vertices: &'a mut Vec<Vertex>,
    pub frame_indices: &'a mut Vec<u16>,
    pub scene: &'a Scene,
    pub camera: &'a Camera,
    pub current_instance: &'a HullInstance,
    pub blueprint_side: &'a BlueprintSide,
    pub side_config: &'a HandlerConfig,
    pub transform_to_camera_host_hull: &'a Mat4,
    pub camera_view_from_host_hull: &'a Mat4,
    pub screen_width: f32,
    pub screen_height: f32,
    pub visible_screen_polygon: ConvexPolygon,
    pub traversal_queue: &'a mut VecDeque<TraversalState>,
    pub current_recursion_depth: u32,
}

pub trait SideHandler: Send + Sync {
    fn process_render(&self, ctx: &mut HandlerContext);
}

pub struct StandardWallHandler;
impl SideHandler for StandardWallHandler {
    fn process_render(&self, ctx: &mut HandlerContext) {
        let wall_color = match ctx.side_config {
            HandlerConfig::StandardWall { color, .. } => *color,
            _ => [0.7, 0.7, 0.7, 1.0],
        };
        if ctx.visible_screen_polygon.count() >= 3 {
            let start_vertex_index = ctx.frame_vertices.len() as u16;
            for point in ctx.visible_screen_polygon.vertices() {
                ctx.frame_vertices.push(Vertex::new([point.x, point.y], wall_color));
            }
            for i in 1..(ctx.visible_screen_polygon.count() as u16 - 1) {
                ctx.frame_indices.push(start_vertex_index);
                ctx.frame_indices.push(start_vertex_index + i);
                ctx.frame_indices.push(start_vertex_index + i + 1);
            }
        }
    }
}

pub fn get_portal_alignment_transform(
    source_portal_id_on_current_bp: PortalId,
    target_portal_id_on_target_bp: PortalId,
) -> Mat4 {
    let room_half_size = 1.5;

    match (source_portal_id_on_current_bp, target_portal_id_on_target_bp) {
        (PORTAL_ID_FRONT, PORTAL_ID_BACK) => {
            Mat4::from_translation(Vec3::new(0.0, 0.0, room_half_size * 2.0))
        }
        (PORTAL_ID_BACK, PORTAL_ID_FRONT) => {
            Mat4::from_translation(Vec3::new(0.0, 0.0, -room_half_size * 2.0))
        }
        (PORTAL_ID_RIGHT, PORTAL_ID_LEFT) => {
            Mat4::from_translation(Vec3::new(room_half_size * 2.0, 0.0, 0.0))
        }
        (PORTAL_ID_LEFT, PORTAL_ID_RIGHT) => {
            Mat4::from_translation(Vec3::new(-room_half_size * 2.0, 0.0, 0.0))
        }
        (PORTAL_ID_TOP, PORTAL_ID_BOTTOM) => {
            Mat4::from_translation(Vec3::new(0.0, room_half_size * 2.0, 0.0))
        }
        (PORTAL_ID_BOTTOM, PORTAL_ID_TOP) => {
            Mat4::from_translation(Vec3::new(0.0, -room_half_size * 2.0, 0.0))
        }
        _ => {
            Mat4::IDENTITY
        }
    }
}

pub struct StandardPortalHandler;
impl SideHandler for StandardPortalHandler {
    fn process_render(&self, ctx: &mut HandlerContext) {
        let (target_instance_id_from_config, target_portal_id_on_target_bp_from_config) = match ctx.side_config {
            HandlerConfig::StandardPortal { target_instance_id, target_portal_id } => (target_instance_id, target_portal_id),
            _ => { return; }
        };

        let portal_local_normal_vec = ctx.blueprint_side.local_normal;

        // Calculate normal_in_cam_space
        let normal_in_host_bp_space = ctx.transform_to_camera_host_hull.transform_vector3(portal_local_normal_vec).normalize_or_zero();
        let normal_in_cam_space = ctx.camera_view_from_host_hull.transform_vector3(normal_in_host_bp_space).normalize_or_zero();

        // --- New Culling Logic ---
        // Get a point on the portal plane in blueprint local space
        if ctx.blueprint_side.vertex_indices.is_empty() {
            // This side has no vertices, cannot be a portal plane
            return;
        }
        let p0_bp_local_idx = ctx.blueprint_side.vertex_indices[0];
        
        // Access blueprint through scene context to get local vertices
        let p0_bp_local = match ctx.scene.blueprints.get(&ctx.current_instance.blueprint_id) {
            Some(blueprint) => {
                if p0_bp_local_idx < blueprint.local_vertices.len() {
                    blueprint.local_vertices[p0_bp_local_idx]
                } else {
                    // Invalid vertex index for blueprint
                    return; 
                }
            }
            None => {
                // Blueprint not found in scene, should not happen
                return; 
            }
        };

        // Transform P0 to camera space
        let p0_host_hull_space = ctx.transform_to_camera_host_hull.transform_point3(p0_bp_local);
        let p0_cam_space = ctx.camera_view_from_host_hull.transform_point3(p0_host_hull_space);

        let d_plane_constant = -normal_in_cam_space.dot(p0_cam_space);

        let culling_epsilon = 1e-5; 
        if d_plane_constant < -culling_epsilon {
            return; // Cull
        }

        // Original culling logic (for reference, now replaced):
        // let cull_threshold_z = 1e-3;
        // if normal_in_cam_space.z <= cull_threshold_z {
        //     return;
        // }

        if ctx.current_recursion_depth >= MAX_PORTAL_RECURSION_DEPTH { return; }
        if !ctx.scene.instances.contains_key(target_instance_id_from_config) { return; }

        let portal_alignment_transform = get_portal_alignment_transform(
            ctx.blueprint_side.local_portal_id.expect("Portal handler on side with no local_portal_id"),
            *target_portal_id_on_target_bp_from_config
        );
        
        let next_transform_to_camera_host_hull = *ctx.transform_to_camera_host_hull * portal_alignment_transform;

        ctx.traversal_queue.push_back(TraversalState {
            current_instance_id: *target_instance_id_from_config,
            accumulated_transform: next_transform_to_camera_host_hull,
            screen_space_clip_polygon: ctx.visible_screen_polygon.clone(),
            recursion_depth: ctx.current_recursion_depth + 1,
        });
    }
}