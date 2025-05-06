use std::sync::Arc;
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
    dpi::PhysicalSize,
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use femtovg::{renderer::OpenGl, Canvas};
use std::num::NonZeroU32;
use std::ffi::CString;
use crate::logging::UI_NAMESPACE;
use log::{debug, info, warn};

pub struct AppWindow {
    pub window: Arc<Window>,
    pub gl_config: glutin::config::Config,
}

impl AppWindow {
    pub fn new(event_loop: &EventLoopWindowTarget<()>) -> Self {
        info!(target: UI_NAMESPACE, "Creating window builder...");
        let window_builder = WindowBuilder::new()
            .with_title("VX220 Dashboard")
            .with_inner_size(PhysicalSize::new(800, 600))
            .with_resizable(true)
            .with_visible(true)
            .with_decorations(true);

        info!(target: UI_NAMESPACE, "Creating GL config template...");
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_stencil_size(8)
            .with_depth_size(24)
            .with_transparency(true);

        info!(target: UI_NAMESPACE, "Creating display builder...");
        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        info!(target: UI_NAMESPACE, "Building display...");
        let (window, gl_config) = display_builder
            .build(event_loop, template, |mut configs| configs.next().unwrap())
            .expect("Failed to create display");

        let window = window.expect("Failed to create window");
        info!(target: UI_NAMESPACE, "Window created with size: {}x{}", window.inner_size().width, window.inner_size().height);

        Self {
            window: Arc::new(window),
            gl_config,
        }
    }
}

pub struct FemtovgContext {
    pub canvas: Canvas<OpenGl>,
    pub surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub gl_context: PossiblyCurrentContext,
}

pub fn create_femtovg_context(app_window: &AppWindow) -> FemtovgContext {
    info!(target: UI_NAMESPACE, "Creating OpenGL context...");
    
    let raw_window_handle = app_window.window.raw_window_handle();

    let context_attributes = ContextAttributesBuilder::new()
        .with_profile(glutin::context::GlProfile::Core)
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
        .with_debug(true)
        .build(Some(raw_window_handle));

    info!(target: UI_NAMESPACE, "Creating GL context...");
    let not_current_context = unsafe {
        app_window.gl_config.display()
            .create_context(&app_window.gl_config, &context_attributes)
            .expect("Failed to create GL context")
    };

    let size = app_window.window.inner_size();
    debug!(target: UI_NAMESPACE, "Window size for surface: {}x{}", size.width, size.height);
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            raw_window_handle,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

    info!(target: UI_NAMESPACE, "Creating surface...");
    let surface = unsafe {
        app_window.gl_config.display()
            .create_window_surface(&app_window.gl_config, &attrs)
            .expect("Failed to create surface")
    };

    info!(target: UI_NAMESPACE, "Making context current...");
    let gl_context = not_current_context
        .make_current(&surface)
        .expect("Failed to make context current");

    // Load GL functions
    info!(target: UI_NAMESPACE, "Loading GL functions...");
    unsafe {
        gl::load_with(|s| {
            let cstr = CString::new(s).unwrap();
            app_window.gl_config.display().get_proc_address(&cstr).cast()
        });

        // Set up OpenGL state
        gl::ClearColor(0.2, 0.2, 0.4, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Viewport(0, 0, size.width as i32, size.height as i32);
    }

    info!(target: UI_NAMESPACE, "Creating renderer...");
    let renderer = unsafe {
        OpenGl::new_from_function_cstr(|s| app_window.gl_config.display().get_proc_address(s).cast())
            .expect("Cannot create renderer")
    };

    info!(target: UI_NAMESPACE, "Creating canvas...");
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(size.width, size.height, app_window.window.scale_factor() as f32);

    // Try to load system fonts
    let font_paths = [
        "C:\\Windows\\Fonts\\segoe.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\verdana.ttf",
    ];

    let mut font_loaded = false;
    for path in font_paths.iter() {
        if let Ok(font_data) = std::fs::read(path) {
            if canvas.add_font_mem(&font_data).is_ok() {
                font_loaded = true;
                info!(target: UI_NAMESPACE, "Successfully loaded font from: {}", path);
                break;
            }
        }
    }

    if !font_loaded {
        warn!(target: UI_NAMESPACE, "No system fonts could be loaded. Text rendering may not work correctly.");
    }

    info!(target: UI_NAMESPACE, "Femtovg context created successfully!");
    surface.swap_buffers(&gl_context).expect("Failed to swap buffers");

    FemtovgContext {
        canvas,
        surface,
        gl_context,
    }
} 