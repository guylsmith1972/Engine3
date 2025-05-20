// src/app.rs

use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

// Use items from our library crate "convex_polygon_intersection"
use convex_polygon_intersection::geometry::{ConvexPolygon, Point2, MAX_VERTICES};
use convex_polygon_intersection::vertex::Vertex;
use convex_polygon_intersection::intersection::ConvexIntersection;
// PolygonGenerator is not directly used by portal renderer MVP
// use convex_polygon_intersection::generator::PolygonGenerator;

// Local modules
use crate::ui::build_ui;
use crate::shader::WGSL_SHADER_SOURCE;
use crate::scene::{Scene, SceneSide, Point3, TraversalState, create_mvp_scene};
use crate::camera::{Camera};

use std::collections::VecDeque; // For traversal queue


pub struct PolygonApp {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    // --- Portal Rendering Specific ---
    scene: Scene,
    camera: Camera,
    // --- End Portal Rendering Specific ---

    // Debug polygons (can be removed or repurposed)
    // polygon1: ConvexPolygon,
    // polygon2: ConvexPolygon,
    // intersection_poly: ConvexPolygon,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    
    // For collecting render data per frame
    frame_vertices: Vec<Vertex>,
    frame_indices: Vec<u16>,


    // animation_time: f32, // Not used in portal MVP
    // is_animating: bool,   // Not used in portal MVP
    // regenerate_requested: bool, // Not used in portal MVP

    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    // Simple camera controls
    camera_pos_delta: Point3,
    camera_yaw_delta: f32,
    camera_pitch_delta: f32,
}

impl PolygonApp {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // Potentially request more limits if needed for larger scenes
                    required_limits: wgpu::Limits::default(), 
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // Consider Mailbox or Immediate for lower latency
            alpha_mode: surface_caps.alpha_modes[0], // Ensure this supports transparency if needed
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(WGSL_SHADER_SOURCE.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[], // No bind groups for this simple shader
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()], // Vertex description from vertex.rs
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Keep alpha blending
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Ensure consistent with polygon winding
                cull_mode: None, // No GPU culling, CPU portal/backface culling
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // TODO: Add depth buffer for correct intra-hull rendering
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Adjust buffer sizes if necessary for more complex scenes
        // MAX_VERTICES is per polygon, MAX_TOTAL_VERTICES_APP is for the whole scene buffer
        // Max sides per hull * max vertices per side * approx max visible hulls
        const MAX_HULLS_VISIBLE_ESTIMATE: usize = 10;
        const MAX_SIDES_PER_HULL_ESTIMATE: usize = 6; // Cube
        const MAX_TOTAL_VERTICES_APP: usize = MAX_VERTICES * MAX_SIDES_PER_HULL_ESTIMATE * MAX_HULLS_VISIBLE_ESTIMATE;
        const MAX_TOTAL_INDICES_APP: usize = (MAX_VERTICES.saturating_sub(2)) * 3 * MAX_SIDES_PER_HULL_ESTIMATE * MAX_HULLS_VISIBLE_ESTIMATE;


        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (MAX_TOTAL_VERTICES_APP * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (MAX_TOTAL_INDICES_APP * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            Some(device.limits().max_texture_dimension_2d as usize), // Use full limit
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            None, // No depth texture for egui in this setup
            1,    // samples
        );

        // --- Portal Rendering Init ---
        let scene = create_mvp_scene();
        let camera = Camera::new(
            Point3::new(0.0, 0.0, 0.0), // Camera at origin
            0.0,  // Yaw (degrees) - looking down -Z
            0.0,  // Pitch (degrees)
            75.0, // FOV Y (degrees)
            0.1,  // ZNear
            100.0 // ZFar
        );
        // --- End Portal Rendering Init ---

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            scene,
            camera,
            vertex_buffer,
            index_buffer,
            frame_vertices: Vec::with_capacity(MAX_TOTAL_VERTICES_APP),
            frame_indices: Vec::with_capacity(MAX_TOTAL_INDICES_APP),
            // animation_time: 0.0,
            // is_animating: false,
            // regenerate_requested: false,
            egui_ctx,
            egui_state,
            egui_renderer,
            camera_pos_delta: Point3::new(0.0,0.0,0.0),
            camera_yaw_delta: 0.0,
            camera_pitch_delta: 0.0,
        }
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
    
    // Triangulates a 2D screen-space polygon and adds its vertices/indices
    // to the frame's render data.
    fn add_polygon_to_frame(
        polygon_2d: &ConvexPolygon, // Clipped screen-space polygon
        color: [f32; 4],
        frame_vertices: &mut Vec<Vertex>,
        frame_indices: &mut Vec<u16>,
    ) {
        if polygon_2d.count() < 3 {
            return;
        }

        let start_vertex_index = frame_vertices.len() as u16;

        for point in polygon_2d.vertices() {
            frame_vertices.push(Vertex::new([point.x, point.y], color));
        }

        // Simple fan triangulation
        for i in 1..(polygon_2d.count() as u16 - 1) {
            frame_indices.push(start_vertex_index);
            frame_indices.push(start_vertex_index + i);
            frame_indices.push(start_vertex_index + i + 1);
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // If using a depth texture, it would need to be resized here too.
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Update camera position based on delta
        let move_speed = 2.0 * dt; // units per second
        let rot_speed = 1.5 * dt; // radians per second

        // Forward/backward movement (along camera's Z axis)
        let forward_dir = Point3::new(-self.camera.yaw.sin() * self.camera.pitch.cos(), self.camera.pitch.sin(), -self.camera.yaw.cos() * self.camera.pitch.cos()).normalize();
        // Strafe movement (along camera's X axis)
        let right_dir = Point3::new(forward_dir.z, 0.0, -forward_dir.x).normalize();


        self.camera.position.x += (forward_dir.x * self.camera_pos_delta.z + right_dir.x * self.camera_pos_delta.x) * move_speed;
        self.camera.position.y += (forward_dir.y * self.camera_pos_delta.z) * move_speed + self.camera_pos_delta.y * move_speed; // Simple up/down
        self.camera.position.z += (forward_dir.z * self.camera_pos_delta.z + right_dir.z * self.camera_pos_delta.x) * move_speed;


        self.camera.yaw += self.camera_yaw_delta * rot_speed;
        self.camera.pitch += self.camera_pitch_delta * rot_speed;

        // Clamp pitch
        self.camera.pitch = self.camera.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

        // Reset deltas
        self.camera_pos_delta = Point3::new(0.0,0.0,0.0);
        self.camera_yaw_delta = 0.0;
        self.camera_pitch_delta = 0.0;
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Clear frame data
        self.frame_vertices.clear();
        self.frame_indices.clear();

        // --- Portal Rendering Logic ---
        let mut traversal_queue: VecDeque<TraversalState> = VecDeque::new();
        
        // Initial screen clip polygon (full screen)
        // Coordinates are (0,0) top-left to (width, height) bottom-right, matching shader expectations
        let initial_clip_points = [
            Point2::new(0.0, 0.0),
            Point2::new(self.size.width as f32, 0.0),
            Point2::new(self.size.width as f32, self.size.height as f32),
            Point2::new(0.0, self.size.height as f32),
        ];
        let initial_clip_polygon = ConvexPolygon::from_points(&initial_clip_points);

        // Determine starting hull (for MVP, assume hull 0 if POV is inside, or simple visibility check)
        // For now, always start with hull 0. A proper implementation would find which hull camera is in.
        let start_hull_id = 0; 

        if !self.scene.hulls.is_empty() {
             traversal_queue.push_back(TraversalState {
                hull_id: start_hull_id,
                screen_space_clip_polygon: initial_clip_polygon,
            });
        }

        let mut processed_hulls_this_frame: std::collections::HashSet<usize> = std::collections::HashSet::new();


        while let Some(current_state) = traversal_queue.pop_front() {
            if processed_hulls_this_frame.contains(&current_state.hull_id) && traversal_queue.len() > 10 { // Simple cycle break for deep recursions
                 //continue; // This check might be too aggressive or needs better state for hashing (clip poly)
            }
            processed_hulls_this_frame.insert(current_state.hull_id);


            let current_hull = match self.scene.hulls.get(current_state.hull_id) {
                Some(hull) => hull,
                None => continue, // Invalid hull ID
            };

            let v_current = &current_state.screen_space_clip_polygon;

            for side in &current_hull.sides {
                // 1. Back-face Culling (normal should point outwards from the hull)
                // Vector from a point on the side to the camera
                if side.vertices_3d.is_empty() { continue; }
                let point_on_side = side.vertices_3d[0];
                let cam_to_side_vec = point_on_side.sub(&self.camera.position);
                if cam_to_side_vec.dot(&side.normal) >= -1e-3 { // Use a small epsilon; >=0 means facing away or parallel
                    continue;
                }

                // 2. Project 3D side to 2D screen space polygon (P_projected)
                let mut projected_points_2d: Vec<Point2> = Vec::with_capacity(side.vertices_3d.len());
                let mut all_points_valid = true;
                for v3d in &side.vertices_3d {
                    if let Some(p2d) = self.camera.project_to_screen(v3d, self.size.width as f32, self.size.height as f32) {
                        projected_points_2d.push(p2d);
                    } else {
                        all_points_valid = false; // Point was behind camera or failed projection
                        break;
                    }
                }
                
                if !all_points_valid || projected_points_2d.len() < 3 {
                    continue; // Cannot form a polygon
                }
                
                let p_projected = ConvexPolygon::from_points(&projected_points_2d);
                if p_projected.count() < 3 { continue; }


                if side.is_portal {
                    if let Some(next_hull_id) = side.connected_hull_id {
                        let mut v_next = ConvexPolygon::new();
                        ConvexIntersection::find_intersection_into(v_current, &p_projected, &mut v_next);
                        
                        if v_next.count() >= 3 {
                            // Basic check to prevent re-queuing the same hull immediately unless it's through a different portal context
                            // A more robust solution would involve comparing the clip polygon too.
                            if !traversal_queue.iter().any(|s| s.hull_id == next_hull_id) || processed_hulls_this_frame.len() < self.scene.hulls.len() {
                                traversal_queue.push_back(TraversalState {
                                    hull_id: next_hull_id,
                                    screen_space_clip_polygon: v_next,
                                });
                            }
                        }
                    }
                } else { // It's a renderable wall
                    let mut clipped_wall_poly = ConvexPolygon::new();
                    ConvexIntersection::find_intersection_into(&p_projected, v_current, &mut clipped_wall_poly);

                    if clipped_wall_poly.count() >= 3 {
                        Self::add_polygon_to_frame(
                            &clipped_wall_poly,
                            side.color,
                            &mut self.frame_vertices,
                            &mut self.frame_indices,
                        );
                    }
                }
            }
        }
        // --- End Portal Rendering Logic ---


        // Update GPU buffers if there's anything to render
        if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
            // Ensure buffer capacity (this is a simplified check, proper resizing is complex)
            if (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as u64 > self.vertex_buffer.size() ||
               (self.frame_indices.len() * std::mem::size_of::<u16>()) as u64 > self.index_buffer.size() {
                eprintln!("Warning: Frame data exceeds buffer capacity. Truncating or erroring.");
                // For MVP, we might just truncate or panic. A real app would resize buffers.
                // For now, let's try to proceed but it might crash if over limit after padding.
            }
            
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.frame_vertices),
            );

            // Index buffer data needs to be padded to a multiple of wgpu::COPY_BUFFER_ALIGNMENT (4 bytes) if not already.
            // For u16, this means if count is odd, add a dummy 0.
            let mut padded_indices_data = self.frame_indices.clone();
            if padded_indices_data.len() % 2 == 1 {
                 padded_indices_data.push(0); // Pad if odd number of u16 indices
            }
            self.queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&padded_indices_data),
            );
        }


        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        { // Main render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05, g: 0.05, b: 0.1, a: 1.0, // Dark blue background
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // TODO: Add depth buffer
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            
            if !self.frame_vertices.is_empty() && !self.frame_indices.is_empty() {
                 let vertex_buffer_slice_size = (self.frame_vertices.len() * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress;
                 let effective_indices_count = self.frame_indices.len(); // Use original count for draw call

                 render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..vertex_buffer_slice_size));
                 
                 // Index buffer slice size needs to account for padding if done during write_buffer
                 let index_buffer_slice_size = if self.frame_indices.len() % 2 == 1 {
                     ((self.frame_indices.len() + 1) * std::mem::size_of::<u16>()) as wgpu::BufferAddress
                 } else {
                     (self.frame_indices.len() * std::mem::size_of::<u16>()) as wgpu::BufferAddress
                 };

                 render_pass.set_index_buffer(self.index_buffer.slice(..index_buffer_slice_size), wgpu::IndexFormat::Uint16);
                 render_pass.draw_indexed(0..effective_indices_count as u32, 0, 0..1);
            }
        }

        // Egui rendering
        let raw_input = self.egui_state.take_egui_input(&window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            build_ui( // build_ui now needs to know what to display, placeholder for now
                ctx,
                // Pass dummy polygons or relevant portal stats for UI
                &ConvexPolygon::new(), 
                &ConvexPolygon::new(),
                &ConvexPolygon::new(),
                &mut false, // &mut self.is_animating,
                &mut false, // &mut self.regenerate_requested,
            );
        });

        self.egui_state.handle_platform_output(window, full_output.platform_output);

        let tris = self.egui_ctx.tessellate(full_output.shapes, self.egui_ctx.pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &tris, &screen_descriptor);

        { // GUI render pass
            let mut gui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load previous pass's content
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.egui_renderer.render(&mut gui_render_pass, &tris, &screen_descriptor);
        }
        
        for tex_id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(tex_id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output_texture.present();

        Ok(())
    }
    

    // This function is no longer used by the portal renderer directly.
    // Kept for reference or if you want to draw isolated polygons for debugging.
    #[allow(dead_code)] 
    fn _update_debug_buffers(&mut self, poly1: &ConvexPolygon, poly2: &ConvexPolygon, intersection: &ConvexPolygon) {
        let mut vertices_data = Vec::new();
        let mut indices_data = Vec::new();

        // Re-implement triangulation or adapt Self::add_polygon_to_frame if needed for debug
        // For now, this function is effectively disabled for the portal MVP main path.
        Self::add_polygon_to_frame(poly1, [1.0, 0.0, 0.0, 0.5], &mut vertices_data, &mut indices_data);
        Self::add_polygon_to_frame(poly2, [0.0, 0.0, 1.0, 0.5], &mut vertices_data, &mut indices_data);
        Self::add_polygon_to_frame(intersection, [0.0, 1.0, 0.0, 0.8], &mut vertices_data, &mut indices_data);


        if !vertices_data.is_empty() {
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices_data),
            );
        }
        if !indices_data.is_empty() {
             let mut padded_indices_data = indices_data.clone();
            if padded_indices_data.len() % 2 == 1 {
                padded_indices_data.push(0); 
            }
            self.queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&padded_indices_data),
            );
        }
    }


    pub fn handle_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        let response = self.egui_state.on_window_event(window, event);
        if response.consumed {
            return true;
        }

        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let pressed = key_event.state == ElementState::Pressed;
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.camera_pos_delta.z = if pressed { -1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.camera_pos_delta.z = if pressed { 1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.camera_pos_delta.x = if pressed { -1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.camera_pos_delta.x = if pressed { 1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::Space) => { // Move up
                        self.camera_pos_delta.y = if pressed { 1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ControlLeft) => { // Move down
                        self.camera_pos_delta.y = if pressed { -1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => {
                        self.camera_yaw_delta = if pressed { 1.0 } else { 0.0 }; true // Turn left
                    }
                    PhysicalKey::Code(KeyCode::ArrowRight) => {
                        self.camera_yaw_delta = if pressed { -1.0 } else { 0.0 }; true // Turn right
                    }
                     PhysicalKey::Code(KeyCode::ArrowUp) => {
                        self.camera_pitch_delta = if pressed { 1.0 } else { 0.0 }; true // Look up
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown) => {
                        self.camera_pitch_delta = if pressed { -1.0 } else { 0.0 }; true // Look down
                    }
                    // Old controls - can be removed or repurposed
                    // PhysicalKey::Code(KeyCode::KeyS) => { self.print_stats(); true }
                    // PhysicalKey::Code(KeyCode::KeyT) => { self.run_performance_test(); true }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    // These functions are from the old 2D demo and not directly relevant to portal MVP.
    // They can be removed or adapted later.
    #[allow(dead_code)]
    fn generate_new_polygons(&mut self) { /* old logic */ }
    #[allow(dead_code)]
    fn print_stats(&self) { /* old logic */ }
    #[allow(dead_code)]
    fn run_performance_test(&mut self) { /* old logic */ }
}