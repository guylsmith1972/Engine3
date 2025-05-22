// src/main.rs

pub mod app;
pub mod ui;
pub mod engine_lib;
pub mod rendering_lib;
pub mod demo_scene;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget, EventLoop}, 
    window::WindowBuilder,
};
use app::PolygonApp;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = std::sync::Arc::new(
        WindowBuilder::new()
            .with_title("Portal Rendering - Refactored")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-viewport")?;
                let canvas = web_sys::Element::from(window.canvas().ok()?); 
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut app_state = PolygonApp::new(window.clone()).await;
    let mut last_time = std::time::Instant::now();

    event_loop
        .run(move |event, target: &EventLoopWindowTarget<()>| { 
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !app_state.handle_window_event(event, &window) {
                        match event {
                            WindowEvent::CloseRequested => {
                                target.exit();
                            }
                            WindowEvent::Resized(physical_size) => {
                                app_state.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => { /* In AboutToWait */ }
                            WindowEvent::Focused(is_focused) => {
                                app_state.set_focused(*is_focused);
                            }
                            _ => {}
                        }
                    }
                }
                Event::DeviceEvent { event: device_event, .. } => {
                    app_state.handle_device_event(&device_event, &window);
                }
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    app_state.update(dt);
                    match app_state.render(&window) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            app_state.resize(app_state.get_size());
                        }
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
                Event::LoopExiting => {}
                _ => {}
            }
        })
        .unwrap();
}

#[tokio::main]
async fn main() {
    run().await;
}