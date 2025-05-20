// src/renderer.rs

use wgpu;
use std::collections::VecDeque;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

// Updated imports to use local library modules and engine_lib types
use super::vertex::Vertex; // From rendering_lib::vertex
use super::geometry::{ConvexPolygon, Point2, MAX_VERTICES}; // From rendering_lib::geometry
use super::intersection::ConvexIntersection; // From rendering_lib::intersection

// Assuming engine_lib is a sibling module in the same crate
use crate::engine_lib::scene_types::{Scene, Point3, TraversalState};
use crate::engine_lib::camera::Camera;


const RENDERER_MAX_VERTICES: usize = MAX_VERTICES * 6 * 10;
const RENDERER_MAX_INDICES: usize = (MAX_VERTICES.saturating_sub(2)) * 3 * 6 * 10;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ScreenDimensionsUniform {
    width: f32,
    height: f32,
    _padding1: f32,
    _padding2: f32,
}

// Helper function for 3D near-plane clipping (Sutherland-Hodgman for one plane)
// Assumes polygon vertices are in camera space.
// Near plane is z_cam = -camera_znear.
// A point (x,y,z) is "inside" (visible) if z < -camera_znear.
fn clip_polygon_near_plane_3d(
    polygon_cam_space: &[Point3],
    camera_znear: f32,
) -> Vec<Point3> {
    if polygon_cam_space.is_empty() {
        return Vec::new();
    }

    let mut output_list = Vec::with_capacity(polygon_cam_space.len() + 1); // Max one extra vertex
    let mut s = polygon_cam_space[polygon_cam_space.len() - 1]; // Start with the last vertex

    for p in polygon_cam_space.iter() {
        let s_is_inside = s.z < -camera_znear;
        let p_is_inside = p.z < -camera_znear;

        if s_is_inside && p_is_inside { // Case 1: Both inside, output P
            output_list.push(*p);
        } else if s_is_inside && !p_is_inside { // Case 2: S inside, P outside, output intersection
            // Calculate intersection point I of edge SP with plane z = -znear
            // I = S + t(P - S)
            // I.z = S.z + t(P.z - S.z) = -camera_znear
            // t = (-camera_znear - S.z) / (P.z - S.z)
            if (p.z - s.z).abs() > 1e-6 { // Avoid division by zero if edge is parallel to plane
                let t = (-camera_znear - s.z) / (p.z - s.z);
                if t >= 0.0 && t <= 1.0 { // Intersection is within the segment
                    let ix = s.x + t * (p.x - s.x);
                    let iy = s.y + t * (p.y - s.y);
                    output_list.push(Point3::new(ix, iy, -camera_znear));
                } else if t < 0.0 && p_is_inside { // s is outside, p is inside, handled in next case (SHOULDN'T HAPPEN with current S,P logic)
                     //This can happen if the segment crosses the plane beyond S, but p is inside.
                     //It implies an issue or that P should have been the start of an exiting segment.
                } else if t > 1.0 && s_is_inside { // s is inside, p is outside, segment crosses beyond p
                    // This should mean the point -camera_znear is not between s.z and p.z, but this is checked by t conditions.
                }

            } else if s_is_inside { // Edge is parallel to plane and inside (or on plane)
                // No intersection to add from this edge crossing, P will be handled next
            }
        } else if !s_is_inside && p_is_inside { // Case 3: S outside, P inside, output I then P
            if (p.z - s.z).abs() > 1e-6 {
                let t = (-camera_znear - s.z) / (p.z - s.z);
                 if t >= 0.0 && t <= 1.0 { // Intersection is within the segment
                    let ix = s.x + t * (p.x - s.x);
                    let iy = s.y + t * (p.y - s.y);
                    output_list.push(Point3::new(ix, iy, -camera_znear));
                }
            }
            output_list.push(*p); // P is inside
        }
        // Case 4: Both S and P are outside, output nothing

        s = *p; // Advance S to P for the next edge
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                buffers: &[Vertex::desc()], // Vertex comes from rendering_lib::vertex
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
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
        }
    }

    fn add_polygon_to_frame(
        &mut self,
        polygon_2d: &ConvexPolygon, // ConvexPolygon comes from rendering_lib::geometry
        color: [f32; 4],
    ) {
        if polygon_2d.count() < 3 { return; }
        let start_vertex_index = self.frame_vertices.len() as u16;
        for point in polygon_2d.vertices() {
            // Vertex comes from rendering_lib::vertex, Point2 from rendering_lib::geometry
            self.frame_vertices.push(Vertex::new([point.x, point.y], color));
        }
        for i in 1..(polygon_2d.count() as u16 - 1) {
            self.frame_indices.push(start_vertex_index);
            self.frame_indices.push(start_vertex_index + i);
            self.frame_indices.push(start_vertex_index + i + 1);
        }
    }

    pub fn render_scene(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        scene: &Scene, // Scene comes from engine_lib::scene_types
        camera: &Camera, // Camera comes from engine_lib::camera
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

        let mut traversal_queue: VecDeque<TraversalState> = VecDeque::new(); // TraversalState from engine_lib::scene_types
        let initial_clip_points = [
            Point2::new(0.0, 0.0), Point2::new(screen_width, 0.0), // Point2 from rendering_lib::geometry
            Point2::new(screen_width, screen_height), Point2::new(0.0, screen_height),
        ];
        // ConvexPolygon from rendering_lib::geometry
        let initial_clip_polygon = ConvexPolygon::from_points(&initial_clip_points);
        let start_hull_id = 0;

        if !scene.hulls.is_empty() {
             traversal_queue.push_back(TraversalState {
                hull_id: start_hull_id, screen_space_clip_polygon: initial_clip_polygon,
            });
        }
        let mut processed_hulls_this_frame: std::collections::HashSet<usize> = std::collections::HashSet::new();
        
        while let Some(current_state) = traversal_queue.pop_front() {
            if processed_hulls_this_frame.contains(&current_state.hull_id) && traversal_queue.len() > scene.hulls.len() * 2 { continue; }
            processed_hulls_this_frame.insert(current_state.hull_id);

            let current_hull = match scene.hulls.get(current_state.hull_id) { Some(h) => h, None => continue };
            let v_current = &current_state.screen_space_clip_polygon;

            for side in &current_hull.sides {
                if side.vertices_3d.is_empty() { continue; }
                let point_on_side = side.vertices_3d[0]; // Point3 from engine_lib::scene_types
                let cam_to_side_vec = point_on_side.sub(&camera.position);
                if cam_to_side_vec.dot(&side.normal) <= 1e-3 { continue; }

                // 1. Transform side vertices to camera space
                let mut vertices_cam_space: Vec<Point3> = Vec::with_capacity(side.vertices_3d.len());
                for v_world in &side.vertices_3d {
                    vertices_cam_space.push(camera.transform_to_camera_space(v_world));
                }

                // 2. Clip polygon against near plane in camera space
                // Point3 from engine_lib::scene_types
                let clipped_vertices_cam_space = clip_polygon_near_plane_3d(&vertices_cam_space, camera.znear);

                if clipped_vertices_cam_space.len() < 3 { continue; }

                // 3. Project clipped camera-space vertices to 2D screen space
                let mut projected_points_2d: Vec<Point2> = Vec::with_capacity(clipped_vertices_cam_space.len());
                // Point2 from rendering_lib::geometry
                for p_cam in &clipped_vertices_cam_space {
                    if let Some(p2d) = camera.project_camera_space_to_screen(p_cam, screen_width, screen_height) {
                        projected_points_2d.push(p2d);
                    } else {
                        // This case should ideally not happen
                    }
                }
                
                if projected_points_2d.len() < 3 { continue; }
                
                // ConvexPolygon from rendering_lib::geometry
                let p_projected_clipped = ConvexPolygon::from_points(&projected_points_2d);
                if p_projected_clipped.count() < 3 { continue; }

                // 4. Proceed with 2D screen-space portal/wall logic
                if side.is_portal {
                    if let Some(next_hull_id) = side.connected_hull_id {
                        let mut v_next = ConvexPolygon::new(); // ConvexPolygon from rendering_lib::geometry
                        // ConvexIntersection from rendering_lib::intersection
                        ConvexIntersection::find_intersection_into(v_current, &p_projected_clipped, &mut v_next);
                        if v_next.count() >= 3 {
                            if !processed_hulls_this_frame.contains(&next_hull_id) || !traversal_queue.iter().any(|s| s.hull_id == next_hull_id && s.screen_space_clip_polygon.vertices() == v_next.vertices()){
                                traversal_queue.push_back(TraversalState { // TraversalState from engine_lib::scene_types
                                    hull_id: next_hull_id, screen_space_clip_polygon: v_next,
                                });
                            }
                        }
                    }
                } else {
                    let mut final_clipped_wall_poly = ConvexPolygon::new(); // ConvexPolygon from rendering_lib::geometry
                    // ConvexIntersection from rendering_lib::intersection
                    ConvexIntersection::find_intersection_into(&p_projected_clipped, v_current, &mut final_clipped_wall_poly);
                    if final_clipped_wall_poly.count() >= 3 {
                        self.add_polygon_to_frame(&final_clipped_wall_poly, side.color);
                    }
                }
            }
        }

        if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
             if (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as u64 > self.vertex_buffer.size() ||
               (self.frame_indices.len() * std::mem::size_of::<u16>()) as u64 > self.index_buffer.size() {
                eprintln!("Renderer Warning: Frame data exceeds pre-allocated buffer capacity.");
            }
            
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.frame_vertices));
            let mut padded_indices_data = self.frame_indices.clone();
            // Ensure data is u8 aligned for webgl
            if padded_indices_data.len() % 2 == 1 { padded_indices_data.push(0); } 
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&padded_indices_data));
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass (Renderer)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, occlusion_query_set: None, timestamp_writes: None,
            });

            if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
                
                let vertex_buffer_slice_size = (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as u64;
                let effective_indices_count = self.frame_indices.len();
                
                // Calculate index buffer slice size correctly for potentially padded data
                let index_buffer_slice_size = if self.frame_indices.len() % 2 == 1 {
                    ((self.frame_indices.len() + 1) * std::mem::size_of::<u16>()) as u64
                } else {
                    (self.frame_indices.len() * std::mem::size_of::<u16>()) as u64
                };


                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..vertex_buffer_slice_size));
                render_pass.set_index_buffer(self.index_buffer.slice(..index_buffer_slice_size), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..effective_indices_count as u32, 0, 0..1);
            }
        }
    }
}