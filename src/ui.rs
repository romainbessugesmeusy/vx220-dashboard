mod window;
mod render;
pub mod widgets;
pub mod theme;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use crate::telemetry::SharedTelemetryState;
use glutin::surface::GlSurface;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use crate::logging::UI_NAMESPACE;
use log::{debug, info, warn};

pub fn run_ui(event_loop: EventLoop<()>, telemetry_state: SharedTelemetryState) {
    info!(target: UI_NAMESPACE, "Creating application window...");
    let app_window = window::AppWindow::new(&event_loop);
    info!(target: UI_NAMESPACE, "Creating femtovg context...");
    let mut femto_ctx = window::create_femtovg_context(&app_window);
    let telemetry_state = telemetry_state.clone();
    let mut last_frame = Instant::now();
    let frame_interval = Duration::from_millis(16); // ~60 FPS
    
    info!(target: UI_NAMESPACE, "Starting event loop...");
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!(target: UI_NAMESPACE, "Window close requested");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    debug!(target: UI_NAMESPACE, "Window resized: {}x{}", size.width, size.height);
                    let width = NonZeroU32::new(size.width.max(1)).unwrap();
                    let height = NonZeroU32::new(size.height.max(1)).unwrap();
                    femto_ctx.surface.resize(&femto_ctx.gl_context, width, height);
                    femto_ctx.canvas.set_size(width.get(), height.get(), app_window.window.scale_factor() as f32);
                    app_window.window.request_redraw();
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                if now.duration_since(last_frame) >= frame_interval {
                    
                    // Render our UI
                    render::render_ui(&mut femto_ctx.canvas, &telemetry_state);
                    
                    // Swap buffers
                    if let Err(e) = femto_ctx.surface.swap_buffers(&femto_ctx.gl_context) {
                        warn!(target: UI_NAMESPACE, "Failed to swap buffers: {:?}", e);
                    }
                    
                    last_frame = now;
                }
                
                // Request next frame
                *control_flow = ControlFlow::WaitUntil(last_frame + frame_interval);
                app_window.window.request_redraw();
            }
            Event::MainEventsCleared => {
                // Only request a redraw if enough time has passed since the last frame
                let now = Instant::now();
                if now.duration_since(last_frame) >= frame_interval {
                    app_window.window.request_redraw();
                }
            }
            _ => {
                *control_flow = ControlFlow::Poll;
            }
        }
    });
}