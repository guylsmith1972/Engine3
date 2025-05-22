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
// Use public demo_scene constants
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

// ADDED this function
pub fn get_portal_alignment_transform(
    source_portal_id_on_current_bp: PortalId,
    target_portal_id_on_target_bp: PortalId,
) -> Mat4 {
    let room_half_size = 1.5; // Matches demo_scene.rs cuboid half_size

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

        let portal_local_normal: &Vec3 = &ctx.blueprint_side.local_normal;
        
        let normal_in_host_bp_space = ctx.transform_to_camera_host_hull.transform_vector3(*portal_local_normal).normalize_or_zero();
        let normal_in_cam_space = ctx.camera_view_from_host_hull.transform_vector3(normal_in_host_bp_space).normalize_or_zero();

        let cull_threshold = 1e-3;
        if normal_in_cam_space.z <= cull_threshold {
            return;
        }

        if ctx.current_recursion_depth >= MAX_PORTAL_RECURSION_DEPTH { return; }
        if !ctx.scene.instances.contains_key(target_instance_id_from_config) { return; }

        // Use the new centralized function
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