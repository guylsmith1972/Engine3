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
    let current_instance_clone = scene.instances.get(&current_instance_id)
         .expect("Active camera instance not found for clone.")
         .clone();
    let current_hull_blueprint = scene.blueprints.get(&current_instance_clone.blueprint_id)
        .expect("Blueprint for active camera instance not found.").clone();

    let boundary_check_result = check_camera_hull_boundary(
        &potential_new_local_pos,
        &current_hull_blueprint,
        &current_instance_clone,
    );

    match boundary_check_result {
        BoundaryCheckResult::Inside => {
            scene.active_camera_local_transform = Mat4::from_translation(potential_new_local_pos) * new_rotation_matrix;
        }
        BoundaryCheckResult::Collision { collided_side_index, collision_point: _ } => { // collision_point is potential_new_local_pos
            let old_position = scene.active_camera_local_transform.w_axis.xyz();
            
            // --- Implement Push Out ---
            let collided_side_normal = current_hull_blueprint.sides[collided_side_index].local_normal;
            
            // We want to move the potential_new_local_pos back along the collided_side_normal
            // so that its distance to the plane is a small positive value (e.g., PUSH_OUT_DISTANCE).
            // Original signed distance for potential_new_local_pos was < -COLLISION_EPSILON.
            // Let current signed_distance = normal.dot(potential_new_local_pos) + d_plane_constant
            // We want new_signed_distance = PUSH_OUT_DISTANCE.
            // The change in position is along 'collided_side_normal'.
            // Let new_pos = potential_new_local_pos + k * collided_side_normal.
            // normal.dot(potential_new_local_pos + k * collided_side_normal) + d_plane_constant = PUSH_OUT_DISTANCE
            // normal.dot(potential_new_local_pos) + d_plane_constant + k * normal.dot(collided_side_normal) = PUSH_OUT_DISTANCE
            // signed_distance + k * (normal.length_squared()) = PUSH_OUT_DISTANCE
            // k = (PUSH_OUT_DISTANCE - signed_distance) / normal.length_squared()

            // Let's recalculate signed_distance for clarity here, or pass it from check_camera_hull_boundary
            let point_on_plane = current_hull_blueprint.local_vertices[current_hull_blueprint.sides[collided_side_index].vertex_indices[0]];
            let d_plane_constant = -collided_side_normal.dot(point_on_plane);
            let signed_distance_at_potential_pos = collided_side_normal.dot(potential_new_local_pos) + d_plane_constant;

            const PUSH_OUT_DISTANCE: f32 = 1e-3; // Small distance to be outside the plane

            let mut corrected_position = potential_new_local_pos;
            if collided_side_normal.length_squared() > 1e-6 { // Avoid division by zero if normal is zero
                // We know signed_distance_at_potential_pos is negative (e.g. -0.001)
                // We want it to be PUSH_OUT_DISTANCE (e.g. 0.001)
                // k = (0.001 - (-0.001)) / len_sq = 0.002 / len_sq
                let k = (PUSH_OUT_DISTANCE - signed_distance_at_potential_pos) / collided_side_normal.length_squared();
                corrected_position = potential_new_local_pos + k * collided_side_normal;
            } else {
                // Normal is zero, unusual. Fallback to old position.
                corrected_position = old_position;
            }
            
            // Sanity check: ensure corrected_position is not further than old_position if movement was small
            // This logic can get complex if multiple collisions happen or if k is very large.
            // For now, a simple push: If the camera intended to move into a wall,
            // place it just outside the wall, but allow rotation.
            // A simpler push: just use the old_position for position component.
            // The more precise push might be better though.

            // Check if the corrected position is "better" than just staying at old_position.
            // If the original movement was tiny, this push might overshoot.
            // A simpler approach for now: just project potential_new_local_pos onto the plane and add a small offset.
            // Projected_pos = P - (N.P + d) * N / N.length_squared()
            // projected_on_plane = potential_new_local_pos - signed_distance_at_potential_pos * collided_side_normal / collided_side_normal.length_squared();
            // corrected_position = projected_on_plane + PUSH_OUT_DISTANCE * collided_side_normal.normalize_or_zero();

            // Sticking to the k-based correction for now:
            scene.active_camera_local_transform = Mat4::from_translation(corrected_position) * new_rotation_matrix;
            
            // Fallback to simpler "just don't move position" if push-out is problematic:
            // scene.active_camera_local_transform = Mat4::from_translation(old_position) * new_rotation_matrix;
        }
        BoundaryCheckResult::Traverse { crossed_side_index, target_instance_id, target_portal_id } => {
            // ... (existing traversal logic) ...
            // Consider adding a PUSH_OUT_DISTANCE equivalent for portal traversal too,
            // to ensure the camera starts slightly *inside* the new room, not exactly on the plane.
            let source_portal_id_on_current_bp = current_hull_blueprint.sides[crossed_side_index]
                .local_portal_id
                .expect("Traversal initiated but source blueprint side has no local_portal_id.");

            let portal_alignment_transform_target_to_current = get_portal_alignment_transform(
                source_portal_id_on_current_bp,
                target_portal_id,
            );

            let camera_pose_if_crossed_in_old_bp = Mat4::from_translation(potential_new_local_pos) * new_rotation_matrix;
            let mut new_camera_pose_in_new_bp = portal_alignment_transform_target_to_current.inverse() * camera_pose_if_crossed_in_old_bp;

            // --- Experimental push into new room ---
            // The "forward" direction for the camera in its new local space is -Z.
            // We want to push it slightly along its new local -Z axis.
            const TRAVERSAL_PUSH_DISTANCE: f32 = 1e-3; // Small push
            let local_push_vec = Vec3::new(0.0, 0.0, -TRAVERSAL_PUSH_DISTANCE); // Along local -Z
            
            // Extract rotation and translation from the new pose
            let (scale, rot_quat, trans) = new_camera_pose_in_new_bp.to_scale_rotation_translation();
            let rotation_matrix_of_new_pose = Mat4::from_quat(rot_quat); // Assuming uniform scale Mat4::from_rotation_translation also works
            
            let world_ish_push_offset = rotation_matrix_of_new_pose.transform_vector3(local_push_vec);
            let pushed_translation = trans + world_ish_push_offset;
            
            new_camera_pose_in_new_bp = Mat4::from_scale_rotation_translation(scale, rot_quat, pushed_translation);
            // --- End experimental push ---


            scene.active_camera_instance_id = target_instance_id;
            scene.active_camera_local_transform = new_camera_pose_in_new_bp;
        }
    }
}