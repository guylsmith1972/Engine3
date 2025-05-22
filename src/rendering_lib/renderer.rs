// src/rendering_lib/renderer.rs

use wgpu;
use std::collections::VecDeque;
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3}; // Added glam import

use super::vertex::Vertex;
use super::geometry::{ConvexPolygon, Point2, MAX_VERTICES};
use super::intersection::ConvexIntersection;

// Refined imports - types needed for direct use or struct fields in this file's logic
use crate::engine_lib::scene_types::{ // Mat4 and Point3 removed from direct import here
    Scene, TraversalState, SideHandlerTypeId, SideIndex
};
use crate::engine_lib::camera::Camera;
use crate::engine_lib::side_handler::{SideHandler, StandardWallHandler, StandardPortalHandler, HandlerContext};


const RENDERER_MAX_VERTICES: usize = MAX_VERTICES * 6 * 20;
const RENDERER_MAX_INDICES: usize = (MAX_VERTICES.saturating_sub(2)) * 3 * 6 * 20;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ScreenDimensionsUniform {
    width: f32,
    height: f32,
    _padding1: f32,
    _padding2: f32,
}

fn clip_polygon_near_plane_3d(
    polygon_cam_space: &[Vec3], // Changed from Point3
    camera_znear: f32,
) -> Vec<Vec3> { // Changed from Point3
    if polygon_cam_space.is_empty() {
        return Vec::new();
    }
    let mut output_list = Vec::with_capacity(polygon_cam_space.len() + 1);
    if polygon_cam_space.is_empty() {
        return output_list;
    }

    let mut s = polygon_cam_space[polygon_cam_space.len() - 1];
    for p_idx in 0..polygon_cam_space.len() {
        let p = &polygon_cam_space[p_idx];
        let s_is_inside = s.z < (-camera_znear + 1e-6);
        let p_is_inside = p.z < (-camera_znear + 1e-6);

        if s_is_inside && p_is_inside {
            output_list.push(*p);
        } else if s_is_inside && !p_is_inside {
            if (p.z - s.z).abs() > 1e-6 {
                let t = (-camera_znear - s.z) / (p.z - s.z);
                if t >= 0.0 && t <= 1.0 {
                    let ix = s.x + t * (p.x - s.x);
                    let iy = s.y + t * (p.y - s.y);
                    output_list.push(Vec3::new(ix, iy, -camera_znear)); // Changed
                }
            }
        } else if !s_is_inside && p_is_inside {
            if (p.z - s.z).abs() > 1e-6 {
                let t = (-camera_znear - s.z) / (p.z - s.z);
                if t >= 0.0 && t <= 1.0 {
                    let ix = s.x + t * (p.x - s.x);
                    let iy = s.y + t * (p.y - s.y);
                    output_list.push(Vec3::new(ix, iy, -camera_znear)); // Changed
                }
            }
            output_list.push(*p);
        }
        s = *p;
    }
    output_list
}

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    frame_vertices: Vec<Vertex>,
    frame_indices: Vec<u16>,
    screen_uniform_buffer: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    wall_handler: Arc<StandardWallHandler>,
    portal_handler: Arc<StandardPortalHandler>,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader_source: &str,
        initial_screen_width: f32,
        initial_screen_height: f32,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Renderer Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let screen_uniform_data = ScreenDimensionsUniform {
            width: initial_screen_width,
            height: initial_screen_height,
            _padding1: 0.0,
            _padding2: 0.0,
        };
        let screen_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Dimensions Uniform Buffer"),
            contents: bytemuck::bytes_of(&screen_uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("screen_dimensions_bind_group_layout"),
        });

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buffer.as_entire_binding(),
            }],
            label: Some("screen_dimensions_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderer Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Renderer Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, 
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Vertex Buffer"),
            size: (RENDERER_MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Index Buffer"),
            size: (RENDERER_MAX_INDICES * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            frame_vertices: Vec::with_capacity(RENDERER_MAX_VERTICES),
            frame_indices: Vec::with_capacity(RENDERER_MAX_INDICES),
            screen_uniform_buffer,
            screen_bind_group,
            wall_handler: Arc::new(StandardWallHandler),
            portal_handler: Arc::new(StandardPortalHandler),
        }
    }

    pub fn render_scene(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        scene: &Scene,
        camera: &Camera,
        screen_width: f32,
        screen_height: f32,
        clear_color: wgpu::Color,
    ) {
        let screen_uniform_data = ScreenDimensionsUniform {
            width: screen_width,
            height: screen_height,
            _padding1: 0.0,
            _padding2: 0.0,
        };
        queue.write_buffer(&self.screen_uniform_buffer, 0, bytemuck::bytes_of(&screen_uniform_data));

        self.frame_vertices.clear();
        self.frame_indices.clear();

        let mut traversal_queue: VecDeque<TraversalState> = VecDeque::new();
        let mut temp_traversal_queue_for_next_depth: VecDeque<TraversalState> = VecDeque::new();

        let initial_clip_points = [
            Point2::new(0.0, 0.0),
            Point2::new(screen_width, 0.0),
            Point2::new(screen_width, screen_height),
            Point2::new(0.0, screen_height),
        ];
        let initial_screen_clip_polygon = ConvexPolygon::from_points(&initial_clip_points);

        let camera_view_from_host_hull = camera.get_view_matrix_from_host_hull(&scene.active_camera_local_transform);

        if scene.instances.contains_key(&scene.active_camera_instance_id) {
            traversal_queue.push_back(TraversalState {
                current_instance_id: scene.active_camera_instance_id,
                accumulated_transform: Mat4::IDENTITY, // Changed
                screen_space_clip_polygon: initial_screen_clip_polygon,
                recursion_depth: 0,
            });
        } else {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass (Error)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, occlusion_query_set: None, timestamp_writes: None,
            });
            return;
        }

        while let Some(current_traversal_state) = traversal_queue.pop_front() {
            let current_instance = match scene.instances.get(&current_traversal_state.current_instance_id) {
                Some(inst) => inst,
                None => continue,
            };
            let blueprint = match scene.blueprints.get(&current_instance.blueprint_id) {
                Some(bp) => bp,
                None => continue,
            };

            for (side_idx, blueprint_side) in blueprint.sides.iter().enumerate() {
                if blueprint_side.vertex_indices.len() < 3 {
                    continue;
                }

                let mut side_vertices_bp_local: Vec<Vec3> = Vec::with_capacity(blueprint_side.vertex_indices.len()); // Changed
                for &v_idx in &blueprint_side.vertex_indices {
                    if v_idx < blueprint.local_vertices.len() {
                        side_vertices_bp_local.push(blueprint.local_vertices[v_idx]);
                    } else {
                        side_vertices_bp_local.clear(); 
                        break;
                    }
                }
                if side_vertices_bp_local.len() < 3 {
                    continue;
                }

                let transform_curr_bp_to_host_bp = &current_traversal_state.accumulated_transform;
                let mut side_vertices_cam_space: Vec<Vec3> = Vec::with_capacity(side_vertices_bp_local.len()); // Changed
                for p_bp_local in &side_vertices_bp_local {
                    // Use transform_point3 for Vec3
                    let p_host_hull_space = transform_curr_bp_to_host_bp.transform_point3(*p_bp_local);
                    side_vertices_cam_space.push(camera_view_from_host_hull.transform_point3(p_host_hull_space));
                }

                let clipped_vertices_cam_space = clip_polygon_near_plane_3d(&side_vertices_cam_space, camera.znear);
                if clipped_vertices_cam_space.len() < 3 {
                    continue;
                }

                let mut projected_points_2d: Vec<Point2> = Vec::with_capacity(clipped_vertices_cam_space.len());
                for p_cam in &clipped_vertices_cam_space {
                    if let Some(p2d) = camera.project_camera_space_to_screen_direct(p_cam, screen_width, screen_height) {
                        projected_points_2d.push(p2d);
                    }
                }
                if projected_points_2d.len() < 3 {
                    continue;
                }

                let p_projected_on_screen = ConvexPolygon::from_points(&projected_points_2d);
                if p_projected_on_screen.count() < 3 {
                    continue;
                }

                let mut final_visible_screen_polygon = ConvexPolygon::new();
                ConvexIntersection::find_intersection_into(
                    &p_projected_on_screen,
                    &current_traversal_state.screen_space_clip_polygon,
                    &mut final_visible_screen_polygon,
                );
                if final_visible_screen_polygon.count() < 3 {
                    continue;
                }

                let side_config_override = current_instance.instance_side_handler_configs.get(&(side_idx as SideIndex));
                let effective_config = side_config_override.unwrap_or(&blueprint_side.default_handler_config);

                let mut handler_ctx = HandlerContext {
                    frame_vertices: &mut self.frame_vertices,
                    frame_indices: &mut self.frame_indices,
                    scene,
                    camera,
                    current_instance,
                    blueprint_side,
                    side_config: effective_config,
                    transform_to_camera_host_hull: &current_traversal_state.accumulated_transform,
                    camera_view_from_host_hull: &camera_view_from_host_hull,
                    screen_width,
                    screen_height,
                    visible_screen_polygon: final_visible_screen_polygon,
                    traversal_queue: &mut temp_traversal_queue_for_next_depth,
                    current_recursion_depth: current_traversal_state.recursion_depth,
                };

                match effective_config.get_intended_handler_type() {
                    SideHandlerTypeId::StandardWall => self.wall_handler.process_render(&mut handler_ctx),
                    SideHandlerTypeId::StandardPortal => self.portal_handler.process_render(&mut handler_ctx),
                    _ => { /* No-op for unhandled types */ }
                }
            }
            traversal_queue.append(&mut temp_traversal_queue_for_next_depth);
        }

        if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.frame_vertices));
            let mut padded_indices_data = self.frame_indices.clone();
            if padded_indices_data.len() % 2 == 1 {
                padded_indices_data.push(0); 
            }
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&padded_indices_data));

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]);

            let vertex_buffer_slice_size = (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as u64;
            let effective_indices_count = self.frame_indices.len();
            let index_buffer_slice_size = if self.frame_indices.len() % 2 == 1 {
                ((self.frame_indices.len() + 1) * std::mem::size_of::<u16>()) as u64
            } else {
                (self.frame_indices.len() * std::mem::size_of::<u16>()) as u64
            };

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..vertex_buffer_slice_size));
            render_pass.set_index_buffer(self.index_buffer.slice(..index_buffer_slice_size), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..effective_indices_count as u32, 0, 0..1);

        } else {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass (Empty Scene)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
    }
}