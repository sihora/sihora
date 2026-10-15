use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use crate::core::a11y::A11yTree;
use crate::core::animation;
use crate::core::{Color, Event, Modifiers};
use crate::render::{DrawCommand, GlyphUpload, Renderer, TextBatch, TextDraw, TextGlyph};
use crate::text::{TextBuffer, TextSystem};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use super::common::process_common_events;

// ---------------------------------------------------------------------------
// Android entry point
// ---------------------------------------------------------------------------

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
std::thread_local! {
    static ANDROID_APP: std::cell::RefCell<Option<AndroidApp>> =
        const { std::cell::RefCell::new(None) };
}

/// Store the Android app handle. Call before `run_app` from `android_main`.
#[cfg(target_os = "android")]
pub fn set_android_app(app: AndroidApp) {
    ANDROID_APP.with(|cell| *cell.borrow_mut() = Some(app));
}

fn create_event_loop() -> EventLoop<()> {
    #[cfg(target_os = "android")]
    {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let app = ANDROID_APP
            .with(|cell| cell.borrow_mut().take())
            .expect("call set_android_app before run_app on Android");
        let mut builder = EventLoop::<()>::builder();
        builder.with_android_app(app);
        builder.build().expect("event loop creation failed")
    }
    #[cfg(not(target_os = "android"))]
    {
        EventLoop::new().expect("event loop creation failed")
    }
}

// ---------------------------------------------------------------------------
// GPU state
// ---------------------------------------------------------------------------

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

// ---------------------------------------------------------------------------
// TextContext — cosmic-text based, same as desktop
// ---------------------------------------------------------------------------

/// Text shaping context for mobile platforms.
pub struct TextContext {
    system: TextSystem,
    uploaded_keys: HashSet<u64>,
}

impl TextContext {
    /// Create a new text context.
    pub fn new() -> Self {
        Self {
            system: TextSystem::new(),
            uploaded_keys: HashSet::new(),
        }
    }

    /// Create a shaped text buffer.
    pub fn create_buffer(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
    ) -> TextBuffer {
        self.system
            .create_buffer(text, font_size, line_height, max_width)
    }

    /// Resize a text buffer's wrapping width.
    pub fn resize_buffer(&mut self, buffer: &mut TextBuffer, max_width: Option<f32>) {
        self.system.resize_buffer(buffer, max_width);
    }

    /// Produce a draw command for a text buffer at the given position.
    pub fn draw_text(&mut self, buffer: &TextBuffer, x: f32, y: f32, color: Color) -> DrawCommand {
        let shaped = self.system.shaped_glyphs(buffer);
        let mut uploads = Vec::new();
        let mut glyphs = Vec::new();

        for g in &shaped {
            if !self.uploaded_keys.contains(&g.cache_key) {
                uploads.push(GlyphUpload {
                    cache_key: g.cache_key,
                    width: g.width,
                    height: g.height,
                    data: g.data.clone(),
                });
                self.uploaded_keys.insert(g.cache_key);
            }
            glyphs.push(TextGlyph {
                x: x + g.x,
                y: y + g.y,
                width: g.width,
                height: g.height,
                cache_key: g.cache_key,
                color,
            });
        }

        DrawCommand::Text(TextBatch { uploads, glyphs })
    }
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDraw for TextContext {
    fn draw_label(
        &mut self,
        text: &str,
        font_size: f32,
        x: f32,
        y: f32,
        color: Color,
    ) -> DrawCommand {
        let buf = self.create_buffer(text, font_size, font_size * 1.2, None);
        self.draw_text(&buf, x, y, color)
    }
}

/// A frame produced by the application: draw commands plus accessibility tree.
pub struct AppFrame {
    /// Draw commands for the renderer.
    pub commands: Vec<DrawCommand>,
    /// Accessibility tree (unused on mobile for now).
    pub a11y: A11yTree,
}

// ---------------------------------------------------------------------------
// MobileApp — touch + lifecycle, no AccessKit
// ---------------------------------------------------------------------------

type AppDrawFn = Box<dyn FnMut(f32, f32, &mut TextContext, &[Event]) -> AppFrame>;

struct MobileApp {
    draw_fn: AppDrawFn,
    text_ctx: TextContext,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    pending_events: Vec<Event>,
    cursor_pos: [f32; 2],
    modifiers: Modifiers,
    last_frame: Option<Instant>,
}

impl MobileApp {
    fn new(draw_fn: AppDrawFn) -> Self {
        Self {
            draw_fn,
            text_ctx: TextContext::new(),
            window: None,
            gpu: None,
            pending_events: Vec::new(),
            cursor_pos: [0.0; 2],
            modifiers: Modifiers::default(),
            last_frame: None,
        }
    }
}

impl ApplicationHandler for MobileApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (window, gpu) = init_gpu(event_loop);
        self.gpu = Some(gpu);
        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // On mobile, the GPU surface becomes invalid on suspend.
        self.gpu = None;
        self.window = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        process_common_events(
            &event,
            &mut self.pending_events,
            &mut self.cursor_pos,
            &mut self.modifiers,
            &self.window,
        );

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = new_size.width.max(1);
                    gpu.config.height = new_size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let (logical_w, logical_h) = match (&self.window, &self.gpu) {
                    (Some(w), Some(g)) => {
                        let s = w.scale_factor() as f32;
                        (g.config.width as f32 / s, g.config.height as f32 / s)
                    }
                    _ => return,
                };

                let dt = self.last_frame.map_or(0.0, |t| t.elapsed().as_secs_f32());
                self.last_frame = Some(Instant::now());

                let events: Vec<Event> = std::mem::take(&mut self.pending_events);
                let frame = (self.draw_fn)(logical_w, logical_h, &mut self.text_ctx, &events);

                // INVARIANT: gpu is Some when window is Some (set together in resumed)
                let gpu = self.gpu.as_mut().expect("gpu state present during redraw");
                render_frame(gpu, &frame.commands, [logical_w, logical_h]);

                if animation::tick(dt) {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn init_gpu(event_loop: &ActiveEventLoop) -> (Arc<Window>, GpuState) {
    let attrs = WindowAttributes::default().with_title("Sihora");

    let window = Arc::new(
        event_loop
            .create_window(attrs)
            .expect("window creation failed"),
    );

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let surface = instance
        .create_surface(window.clone())
        .expect("surface creation failed");

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("no compatible GPU adapter");

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .expect("GPU device creation failed");

    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);

    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let renderer = Renderer::new(&device, format);

    let gpu = GpuState {
        surface,
        device,
        queue,
        config,
        renderer,
    };

    (window, gpu)
}

fn render_frame(gpu: &mut GpuState, commands: &[DrawCommand], viewport: [f32; 2]) {
    let frame = match gpu.surface.get_current_texture() {
        Ok(f) => f,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.surface.configure(&gpu.device, &gpu.config);
            return;
        }
        Err(e) => {
            eprintln!("surface error: {e}");
            return;
        }
    };

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    gpu.renderer
        .render(&gpu.device, &gpu.queue, &view, viewport, commands);
    frame.present();
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run a mobile app. `draw_fn` is called each frame with `(width, height)` in logical pixels.
pub fn run(mut draw_fn: impl FnMut(f32, f32) -> Vec<DrawCommand> + 'static) {
    run_app(move |w, h, _text_ctx, _events| AppFrame {
        commands: draw_fn(w, h),
        a11y: A11yTree::empty("Sihora"),
    });
}

/// Run a mobile app with text support.
pub fn run_with_text(
    mut draw_fn: impl FnMut(f32, f32, &mut TextContext) -> Vec<DrawCommand> + 'static,
) {
    run_app(move |w, h, text_ctx, _events| AppFrame {
        commands: draw_fn(w, h, text_ctx),
        a11y: A11yTree::empty("Sihora"),
    });
}

/// Run a reactive mobile app with text and event support.
pub fn run_reactive(
    mut draw_fn: impl FnMut(f32, f32, &mut TextContext, &[Event]) -> Vec<DrawCommand> + 'static,
) {
    run_app(move |w, h, text_ctx, events| AppFrame {
        commands: draw_fn(w, h, text_ctx, events),
        a11y: A11yTree::empty("Sihora"),
    });
}

/// Run a mobile app with full frame output.
pub fn run_app(draw_fn: impl FnMut(f32, f32, &mut TextContext, &[Event]) -> AppFrame + 'static) {
    let event_loop = create_event_loop();
    let mut app = MobileApp::new(Box::new(draw_fn));
    event_loop.run_app(&mut app).expect("event loop error");
}
