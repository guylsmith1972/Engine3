// src/app.rs

use winit::{
    event::{WindowEvent, DeviceEvent},
    window::{Window, CursorGrabMode},
};
use crate::ui::build_ui;
use crate::rendering_lib::shader::WGSL_SHADER_SOURCE;
use crate::rendering_lib::renderer::Renderer;
use crate::engine_lib::camera::Camera;
use crate::engine_lib::controller::CameraController;
use crate::engine_lib::scene_types::Scene; // Mat4 removed from direct import
use crate::demo_scene;

pub struct PolygonApp {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    renderer: Renderer,
    scene: Scene,
    camera: Camera,
    camera_controller: CameraController,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    is_focused: bool,
}

impl PolygonApp {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
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
        let surface_format = surface_caps.formats.iter().copied()
            .find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        
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
            &device, config.format, WGSL_SHADER_SOURCE,
            size.width as f32, size.height as f32,
        );

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(), egui::ViewportId::ROOT, &window,
            Some(window.scale_factor() as f32),
            None, 
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device, config.format, None, 1,
        );

        let scene = demo_scene::create_mvp_scene();

        let camera = Camera::new(75.0, 0.1, 100.0);
        
        let initial_focus = window.has_focus();
        let mut initial_grab = false;
        if initial_focus {
            if window.set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                .is_ok() {
                window.set_cursor_visible(false);
                initial_grab = true;
            } else { eprintln!("Could not grab cursor on init."); }
        }
        
        // Initialize CameraController with the orientation from demo_scene's initial camera pose
        let initial_cam_yaw_from_scene = std::f32::consts::PI; // Matches demo_scene
        let initial_cam_pitch_from_scene = 0.0;             // Matches demo_scene

        let camera_controller = CameraController::new(
            initial_cam_yaw_from_scene, 
            initial_cam_pitch_from_scene, 
            initial_grab, 
            0.002
        );

        Self {
            surface, device, queue, config, size,
            renderer, scene, camera, camera_controller,
            egui_ctx, egui_state, egui_renderer,
            is_focused: initial_focus,
        }
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> { self.size }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.apply_to_transform(&mut self.scene.active_camera_local_transform, dt);
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Command Encoder"),
        });

        self.renderer.render_scene(
            &self.device, &self.queue, &mut encoder, &view,
            &self.scene, &self.camera,
            self.size.width as f32, self.size.height as f32,
            wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }, 
        );

        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| { build_ui(ctx); });
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
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }, 
                })],
                depth_stencil_attachment: None, occlusion_query_set: None, timestamp_writes: None,
            });
            self.egui_renderer.render(&mut gui_render_pass, &tris, &screen_descriptor);
        }
        for tex_id in &full_output.textures_delta.free { self.egui_renderer.free_texture(tex_id); }

        self.queue.submit(std::iter::once(encoder.finish()));
        output_texture.present();
        Ok(())
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) -> bool {
        if self.egui_state.on_window_event(window, event).consumed { return true; }
        if self.camera_controller.handle_window_event(event, window) { return true; }
        match event {
            WindowEvent::Focused(focused) => { self.is_focused = *focused; false }
            _ => false,
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent, _window: &Window) {
        self.camera_controller.handle_device_event(event);
    }
}