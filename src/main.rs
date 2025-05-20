// src/main.rs

// Modules specific to this binary application
pub mod app;
pub mod shader;
pub mod ui;
pub mod scene;
pub mod camera;
pub mod renderer; // Ensure this line is present and public

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use app::PolygonApp;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    env_logger::init(); 

    let event_loop = EventLoop::new().unwrap();
    let window = std::sync::Arc::new(
        WindowBuilder::new()
            .with_title("Portal Rendering - Encapsulated Renderer") 
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768)) 
            .build(&event_loop)
            .unwrap(),
    );

    let mut app_state = PolygonApp::new(window.clone()).await;
    let mut last_time = std::time::Instant::now();

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll); 

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !app_state.handle_input(event, &window) {
                        match event {
                            WindowEvent::CloseRequested => target.exit(),
                            WindowEvent::Resized(physical_size) => {
                                app_state.resize(*physical_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::AboutToWait => { 
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    app_state.update(dt); 
                    match app_state.render(&window) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => app_state.resize(app_state.get_size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("WGPU Out Of Memory! Exiting.");
                            target.exit();
                        }
                        Err(e) => eprintln!("Surface error: {:?}", e),
                    }
                    if !target.exiting() {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

#[tokio::main]
async fn main() {
    run().await;
}