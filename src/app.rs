// src/app.rs

use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

// Library items
use convex_polygon_intersection::geometry::ConvexPolygon;

// Local modules
use crate::ui::build_ui;
use crate::shader::WGSL_SHADER_SOURCE;
use crate::scene::{Scene, Point3, create_mvp_scene}; // TraversalState is now internal to Renderer
use crate::camera::Camera;
use crate::renderer::Renderer;


pub struct PolygonApp {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    
    renderer: Renderer, 

    scene: Scene,
    camera: Camera,
    
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(
            &device, 
            config.format, 
            WGSL_SHADER_SOURCE,
            size.width as f32,  // Pass initial width
            size.height as f32, // Pass initial height
        );

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            None, 
            1,    
        );

        let scene = create_mvp_scene();
        let camera = Camera::new(
            Point3::new(0.0, 0.0, -2.0), 
            0.0, 0.0, 75.0, 0.1, 100.0
        );

        Self {
            surface,
            device, 
            queue,
            config,
            size,
            renderer, 
            scene,
            camera,
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
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // The renderer's uniform buffer is updated each frame in render_scene,
            // so no explicit renderer.resize() is strictly needed for just screen dims.
        }
    }

    pub fn update(&mut self, dt: f32) {
        let move_speed = 2.0 * dt; 
        let rot_speed = 1.5 * dt; 
        let forward_dir = Point3::new(-self.camera.yaw.sin() * self.camera.pitch.cos(), self.camera.pitch.sin(), -self.camera.yaw.cos() * self.camera.pitch.cos()).normalize();
        let right_dir = Point3::new(forward_dir.z, 0.0, -forward_dir.x).normalize();
        self.camera.position.x += (forward_dir.x * self.camera_pos_delta.z + right_dir.x * self.camera_pos_delta.x) * move_speed;
        self.camera.position.y += (forward_dir.y * self.camera_pos_delta.z) * move_speed + self.camera_pos_delta.y * move_speed; 
        self.camera.position.z += (forward_dir.z * self.camera_pos_delta.z + right_dir.z * self.camera_pos_delta.x) * move_speed;
        self.camera.yaw += self.camera_yaw_delta * rot_speed;
        self.camera.pitch += self.camera_pitch_delta * rot_speed;
        self.camera.pitch = self.camera.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
        self.camera_pos_delta = Point3::new(0.0,0.0,0.0);
        self.camera_yaw_delta = 0.0;
        self.camera_pitch_delta = 0.0;
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Command Encoder"),
        });

        self.renderer.render_scene(
            &self.device, // Pass device for potential future use by renderer if it needs to create temp resources
            &self.queue,
            &mut encoder,
            &view,
            &self.scene,
            &self.camera,
            self.size.width as f32,
            self.size.height as f32,
            wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }, 
        );

        // Egui rendering
        let raw_input = self.egui_state.take_egui_input(&window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            build_ui(ctx, &ConvexPolygon::new(), &ConvexPolygon::new(), &ConvexPolygon::new(), &mut false, &mut false);
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
        { 
            let mut gui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }, 
                })],
                depth_stencil_attachment: None, occlusion_query_set: None, timestamp_writes: None,
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
    
    pub fn handle_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        let response = self.egui_state.on_window_event(window, event);
        if response.consumed { return true; }
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let pressed = key_event.state == ElementState::Pressed;
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => { self.camera_pos_delta.z = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyS) => { self.camera_pos_delta.z = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyA) => { self.camera_pos_delta.x = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyD) => { self.camera_pos_delta.x = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::Space) => { self.camera_pos_delta.y = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ControlLeft) => { self.camera_pos_delta.y = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => { self.camera_yaw_delta = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowRight) => { self.camera_yaw_delta = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowUp) => { self.camera_pitch_delta = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowDown) => { self.camera_pitch_delta = if pressed { -1.0 } else { 0.0 }; true }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}