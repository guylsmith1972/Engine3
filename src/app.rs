// src/app.rs

use winit::{
    event::{ElementState, WindowEvent, DeviceEvent, MouseScrollDelta},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, CursorGrabMode},
};

// Library items
use convex_polygon_intersection::geometry::ConvexPolygon;

// Local modules
use crate::ui::build_ui;
use crate::shader::WGSL_SHADER_SOURCE;
use crate::scene::{Scene, Point3, create_mvp_scene};
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

    // Input state
    camera_pos_delta: Point3,
    camera_yaw_delta: f32,
    camera_pitch_delta: f32,
    mouse_sensitivity: f32,
    is_focused: bool,
    cursor_grabbed: bool,
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
            &device, 
            config.format, 
            WGSL_SHADER_SOURCE,
            size.width as f32,
            size.height as f32,
        );

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(), egui::ViewportId::ROOT, &window,
            Some(window.scale_factor() as f32),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device, config.format, None, 1,    
        );

        let scene = create_mvp_scene(); // Scene colors were updated in the previous step
        let camera = Camera::new(
            Point3::new(0.0, 0.0, -2.0), 
            0.0, 0.0, 75.0, 0.1, 100.0
        );
        
        let initial_focus = window.has_focus();
        let mut cursor_grabbed_on_init = false;
        if initial_focus {
            if window.set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                .is_ok() {
                window.set_cursor_visible(false);
                cursor_grabbed_on_init = true;
            } else {
                eprintln!("Could not grab cursor on init.");
            }
        }

        Self {
            surface, device, queue, config, size,
            renderer, 
            scene, camera,
            egui_ctx, egui_state, egui_renderer,
            camera_pos_delta: Point3::new(0.0,0.0,0.0),
            camera_yaw_delta: 0.0,
            camera_pitch_delta: 0.0,
            mouse_sensitivity: 0.002, 
            is_focused: initial_focus,
            cursor_grabbed: cursor_grabbed_on_init,
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

    fn grab_cursor(&mut self, window: &Window, grab: bool) {
        if grab {
            if !self.cursor_grabbed { 
                window.set_cursor_grab(CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                    .unwrap_or_else(|e| eprintln!("Could not grab cursor: {:?}", e));
                window.set_cursor_visible(false);
                self.cursor_grabbed = true;
            }
        } else {
            if self.cursor_grabbed { 
                 window.set_cursor_grab(CursorGrabMode::None)
                    .unwrap_or_else(|e| eprintln!("Could not ungrab cursor: {:?}", e));
                window.set_cursor_visible(true);
                self.cursor_grabbed = false;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let move_speed = 3.0 * dt; 
        let rot_speed = 1.5 * dt; 

        let cos_pitch = self.camera.pitch.cos();
        let sin_pitch = self.camera.pitch.sin();
        let cos_yaw = self.camera.yaw.cos();
        let sin_yaw = self.camera.yaw.sin();

        let forward_dir = Point3::new(
            -sin_yaw * cos_pitch, 
            sin_pitch,            
            -cos_yaw * cos_pitch  
        ).normalize();
        
        let right_dir = Point3::new(-forward_dir.z, 0.0, forward_dir.x).normalize();
        
        let effective_forward_input = -self.camera_pos_delta.z;
        let effective_strafe_input = self.camera_pos_delta.x;

        let move_vec_fwd = forward_dir.mul_scalar(effective_forward_input * move_speed);
        let move_vec_strafe = right_dir.mul_scalar(effective_strafe_input * move_speed);
        
        self.camera.position = self.camera.position.add(&move_vec_fwd);
        self.camera.position = self.camera.position.add(&move_vec_strafe);
        self.camera.position.y += self.camera_pos_delta.y * move_speed;

        // Apply keyboard rotation
        self.camera.yaw += self.camera_yaw_delta * rot_speed; // Yaw is correct
        self.camera.pitch += self.camera_pitch_delta * rot_speed; // Pitch delta signs adjusted in handle_input

        self.camera.pitch = self.camera.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01, 
            std::f32::consts::FRAC_PI_2 - 0.01
        );

        self.camera_pos_delta = Point3::new(0.0,0.0,0.0);
        self.camera_yaw_delta = 0.0;
        self.camera_pitch_delta = 0.0;
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
                    view: &view, resolve_target: None,
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
        let egui_consumed = self.egui_state.on_window_event(window, event);
        if egui_consumed.consumed { return true; }

        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == ElementState::Pressed && key_event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                     self.grab_cursor(window, !self.cursor_grabbed); 
                     return true; 
                }
                let pressed = key_event.state == ElementState::Pressed;
                match key_event.physical_key {
                    // Translation and Yaw are correct from previous step
                    PhysicalKey::Code(KeyCode::KeyW) => { self.camera_pos_delta.z = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyS) => { self.camera_pos_delta.z = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyA) => { self.camera_pos_delta.x = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyD) => { self.camera_pos_delta.x = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::Space) => { self.camera_pos_delta.y = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ControlLeft) => { 
                        self.camera_pos_delta.y = if pressed { -1.0 } else { 0.0 }; true 
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => { self.camera_yaw_delta = if pressed { 1.0 } else { 0.0 }; true } 
                    PhysicalKey::Code(KeyCode::ArrowRight) => { self.camera_yaw_delta = if pressed { -1.0 } else { 0.0 }; true }
                    
                    // **** MODIFIED Keyboard Pitch delta to fix inversion ****
                    // To make ArrowUp look UP (increase pitch, assuming positive pitch is up):
                    // If visual result of increasing pitch was DOWN, then to look UP, we need to DECREASE pitch.
                    PhysicalKey::Code(KeyCode::ArrowUp) => { self.camera_pitch_delta = if pressed { -1.0 } else { 0.0 }; true } // Was 1.0
                    PhysicalKey::Code(KeyCode::ArrowDown) => { self.camera_pitch_delta = if pressed { 1.0 } else { 0.0 }; true }  // Was -1.0
                    
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if self.is_focused && !self.cursor_grabbed && *state == ElementState::Pressed && *button == winit::event::MouseButton::Left {
                    self.grab_cursor(window, true);
                    return true; 
                }
                false 
            }
            WindowEvent::Focused(focused) => {
                self.is_focused = *focused;
                if !*focused && self.cursor_grabbed { 
                    self.grab_cursor(window, false);
                }
                false
            }
            _ => false,
        }
    }

    pub fn handle_device_input(&mut self, event: &DeviceEvent, _window: &Window) {
        if !self.cursor_grabbed { 
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                // Yaw is correct from previous step
                self.camera.yaw -= *dx as f32 * self.mouse_sensitivity; 
                
                // **** MODIFIED Mouse Pitch to fix inversion ****
                // Mouse Up (dy < 0) should make camera look UP (increase pitch).
                // If visual result of increasing pitch was DOWN, then to look UP, we need to DECREASE pitch.
                // The current mouse logic self.camera.pitch -= *dy * sens;
                //   Mouse Up (dy < 0) => pitch -= (negative) => pitch INCREASES. This was causing look DOWN.
                // To make Mouse Up look UP, we need pitch to DECREASE.
                // So, if dy < 0 (mouse up), we should ADD dy (a negative number) to pitch.
                self.camera.pitch += *dy as f32 * self.mouse_sensitivity; // Was -=
                
                self.camera.pitch = self.camera.pitch.clamp(
                    -std::f32::consts::FRAC_PI_2 + 0.01, 
                    std::f32::consts::FRAC_PI_2 - 0.01
                );
            }
            DeviceEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_x, _y_scroll) = delta {
                     // println!("Mouse Wheel Delta: {:.1}", _y_scroll);
                }
            }
            _ => {}
        }
    }
}