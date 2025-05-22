// src/engine_lib/scene_logic.rs
use glam::{Mat4, Vec3, Vec4Swizzles}; // Added Vec4Swizzles
use crate::engine_lib::scene_types::{
    Scene, HullBlueprint, HullInstance, HandlerConfig,
    SideIndex, BoundaryCheckResult, // Removed unused InstanceId, PortalId
};
use crate::engine_lib::side_handler::get_portal_alignment_transform;

const COLLISION_EPSILON: f32 = 1e-4; // Small epsilon for plane distance

pub fn check_camera_hull_boundary(
    new_camera_pos_in_blueprint_space: &Vec3,
    current_hull_blueprint: &HullBlueprint,
    current_hull_instance: &HullInstance,
) -> BoundaryCheckResult {
    for (side_idx, blueprint_side) in current_hull_blueprint.sides.iter().enumerate() {
        if blueprint_side.vertex_indices.is_empty() {
            continue;
        }
        let point_on_plane = current_hull_blueprint.local_vertices[blueprint_side.vertex_indices[0]];
        let normal = blueprint_side.local_normal;

        let d_plane_constant = -normal.dot(point_on_plane);
        let signed_distance = normal.dot(*new_camera_pos_in_blueprint_space) + d_plane_constant;

        if signed_distance < -COLLISION_EPSILON {
            let handler_config = current_hull_instance
                .instance_side_handler_configs
                .get(&(side_idx as SideIndex))
                .unwrap_or(&blueprint_side.default_handler_config);

            match handler_config {
                HandlerConfig::StandardPortal { target_instance_id, target_portal_id } => {
                    if blueprint_side.local_portal_id.is_some() {
                         return BoundaryCheckResult::Traverse {
                            crossed_side_index: side_idx as SideIndex,
                            target_instance_id: *target_instance_id,
                            target_portal_id: *target_portal_id,
                        };
                    } else {
                        return BoundaryCheckResult::Collision {
                            collided_side_index: side_idx as SideIndex,
                            collision_point: *new_camera_pos_in_blueprint_space,
                        };
                    }
                }
                _ => {
                    return BoundaryCheckResult::Collision {
                        collided_side_index: side_idx as SideIndex,
                        collision_point: *new_camera_pos_in_blueprint_space,
                    };
                }
            }
        }
    }
    BoundaryCheckResult::Inside
}

pub fn update_camera_in_scene(
    scene: &mut Scene,
    potential_new_local_pos: Vec3,
    new_rotation_matrix: Mat4,
    _dt: f32,
) {
    let current_instance_id = scene.active_camera_instance_id;

    let (blueprint_id, _initial_transform_needed_for_bp_lookup) = { // Renamed variable to avoid unused warning
        let instance = scene.instances.get(&current_instance_id)
            .expect("Active camera instance not found in scene.");
        (instance.blueprint_id, instance.initial_transform)
    };
    let current_hull_blueprint = scene.blueprints.get(&blueprint_id)
        .expect("Blueprint for active camera instance not found.").clone();
    
    let current_hull_instance_clone = scene.instances.get(&current_instance_id)
         .expect("Active camera instance not found for clone.")
         .clone();


    let boundary_check_result = check_camera_hull_boundary(
        &potential_new_local_pos,
        &current_hull_blueprint,
        &current_hull_instance_clone,
    );

    match boundary_check_result {
        BoundaryCheckResult::Inside => {
            scene.active_camera_local_transform = Mat4::from_translation(potential_new_local_pos) * new_rotation_matrix;
        }
        BoundaryCheckResult::Collision { collided_side_index: _, collision_point: _ } => {
            let old_position = scene.active_camera_local_transform.w_axis.xyz(); // This line should now work
            scene.active_camera_local_transform = Mat4::from_translation(old_position) * new_rotation_matrix;
        }
        BoundaryCheckResult::Traverse { crossed_side_index, target_instance_id, target_portal_id } => {
            let source_portal_id_on_current_bp = current_hull_blueprint.sides[crossed_side_index]
                .local_portal_id
                .expect("Traversal initiated but source blueprint side has no local_portal_id.");

            let portal_alignment_transform_target_to_current = get_portal_alignment_transform(
                source_portal_id_on_current_bp,
                target_portal_id,
            );

            let camera_pose_if_crossed_in_old_bp = Mat4::from_translation(potential_new_local_pos) * new_rotation_matrix;
            let new_camera_pose_in_new_bp = portal_alignment_transform_target_to_current.inverse() * camera_pose_if_crossed_in_old_bp;

            scene.active_camera_instance_id = target_instance_id;
            scene.active_camera_local_transform = new_camera_pose_in_new_bp;

            // TODO: Adjust CameraController's current_yaw/current_pitch if portal_alignment_transform involved rotation.
        }
    }
}