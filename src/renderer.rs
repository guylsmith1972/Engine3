// src/renderer.rs

use wgpu;
use std::collections::VecDeque;

use crate::scene::{Scene, TraversalState};
use crate::camera::Camera;
use convex_polygon_intersection::geometry::{ConvexPolygon, Point2, MAX_VERTICES};
use convex_polygon_intersection::intersection::ConvexIntersection;
use convex_polygon_intersection::vertex::Vertex;

// Constants for buffer sizes
const RENDERER_MAX_VERTICES: usize = MAX_VERTICES * 6 * 10; // Max verts per polygon * max sides * max hulls
const RENDERER_MAX_INDICES: usize = (MAX_VERTICES.saturating_sub(2)) * 3 * 6 * 10;

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    
    // Internal buffers for accumulating geometry per frame
    frame_vertices: Vec<Vertex>,
    frame_indices: Vec<u16>,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader_source: &str,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Renderer Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Renderer Pipeline Layout"),
                bind_group_layouts: &[],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer Vertex Buffer"),
            size: (RENDERER_MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer Index Buffer"),
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
        }
    }

    fn add_polygon_to_frame(
        &mut self, // Now a method of Renderer
        polygon_2d: &ConvexPolygon, 
        color: [f32; 4],
    ) {
        if polygon_2d.count() < 3 {
            return;
        }
        let start_vertex_index = self.frame_vertices.len() as u16;
        for point in polygon_2d.vertices() {
            self.frame_vertices.push(Vertex::new([point.x, point.y], color));
        }
        for i in 1..(polygon_2d.count() as u16 - 1) {
            self.frame_indices.push(start_vertex_index);
            self.frame_indices.push(start_vertex_index + i);
            self.frame_indices.push(start_vertex_index + i + 1);
        }
    }

    pub fn render_scene(
        &mut self, // Needs mutable access to update frame_vertices/indices
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
        self.frame_vertices.clear();
        self.frame_indices.clear();

        // --- Portal Traversal Logic (MOVED INTO RENDERER) ---
        let mut traversal_queue: VecDeque<TraversalState> = VecDeque::new();
        let initial_clip_points = [
            Point2::new(0.0, 0.0), Point2::new(screen_width, 0.0),
            Point2::new(screen_width, screen_height), Point2::new(0.0, screen_height),
        ];
        let initial_clip_polygon = ConvexPolygon::from_points(&initial_clip_points);
        let start_hull_id = 0; // TODO: Determine start hull based on camera position

        if !scene.hulls.is_empty() {
             traversal_queue.push_back(TraversalState {
                hull_id: start_hull_id, screen_space_clip_polygon: initial_clip_polygon,
            });
        }
        let mut processed_hulls_this_frame: std::collections::HashSet<usize> = std::collections::HashSet::new();
        
        while let Some(current_state) = traversal_queue.pop_front() {
            // Basic cycle/redundancy check (can be improved)
            if processed_hulls_this_frame.contains(&current_state.hull_id) && traversal_queue.len() > scene.hulls.len() * 2 { 
                 continue; 
            }
            processed_hulls_this_frame.insert(current_state.hull_id);

            let current_hull = match scene.hulls.get(current_state.hull_id) { Some(h) => h, None => continue };
            let v_current = &current_state.screen_space_clip_polygon;

            for side in &current_hull.sides {
                if side.vertices_3d.is_empty() { continue; }
                let point_on_side = side.vertices_3d[0];
                let cam_to_side_vec = point_on_side.sub(&camera.position);
                
                if cam_to_side_vec.dot(&side.normal) <= 1e-3 { continue; } // Back-face CULL

                let mut projected_points_2d: Vec<Point2> = Vec::with_capacity(side.vertices_3d.len());
                let mut all_points_valid = true;
                for v3d in &side.vertices_3d {
                    if let Some(p2d) = camera.project_to_screen(v3d, screen_width, screen_height) {
                        projected_points_2d.push(p2d);
                    } else { all_points_valid = false; break; }
                }
                if !all_points_valid || projected_points_2d.len() < 3 { continue; }
                
                let p_projected = ConvexPolygon::from_points(&projected_points_2d);
                if p_projected.count() < 3 { continue; }

                if side.is_portal {
                    if let Some(next_hull_id) = side.connected_hull_id {
                        let mut v_next = ConvexPolygon::new();
                        ConvexIntersection::find_intersection_into(v_current, &p_projected, &mut v_next);
                        if v_next.count() >= 3 {
                             // A more robust check against re-queuing might involve clip polygon comparison
                            if !processed_hulls_this_frame.contains(&next_hull_id) || !traversal_queue.iter().any(|s| s.hull_id == next_hull_id && s.screen_space_clip_polygon.vertices() == v_next.vertices()){
                                traversal_queue.push_back(TraversalState {
                                    hull_id: next_hull_id, screen_space_clip_polygon: v_next,
                                });
                            }
                        }
                    }
                } else { 
                    let mut clipped_wall_poly = ConvexPolygon::new();
                    ConvexIntersection::find_intersection_into(&p_projected, v_current, &mut clipped_wall_poly);
                    if clipped_wall_poly.count() >= 3 {
                        self.add_polygon_to_frame(&clipped_wall_poly, side.color);
                    }
                }
            }
        }
        // --- End Portal Traversal Logic ---

        if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
            if (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as u64 > self.vertex_buffer.size() ||
               (self.frame_indices.len() * std::mem::size_of::<u16>()) as u64 > self.index_buffer.size() {
                eprintln!("Renderer Warning: Frame data exceeds buffer capacity.");
            }
            
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.frame_vertices));
            let mut padded_indices_data = self.frame_indices.clone();
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
            }
        }
    }
}