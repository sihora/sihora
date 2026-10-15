use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use sihora_core::a11y::A11yTree;
use sihora_core::animation;
use sihora_core::{Color, Event, Modifiers};
use sihora_render::{DrawCommand, GlyphUpload, Renderer, TextBatch, TextDraw, TextGlyph};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::common::process_common_events;

// ---------------------------------------------------------------------------
// TextContext — uses Canvas 2D for text rasterization on web
// ---------------------------------------------------------------------------

/// Text rendering context for the web platform.
///
/// Uses an offscreen Canvas 2D to rasterize text into glyph bitmaps that are
/// uploaded to the GPU atlas.
pub struct TextContext {
    text_canvas: HtmlCanvasElement,
    text_ctx_2d: CanvasRenderingContext2d,
    uploaded_keys: HashSet<u64>,
}

impl TextContext {
    /// Create a new web text context.
    pub fn new() -> Self {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .expect("create canvas")
            .dyn_into()
            .expect("cast canvas");
        canvas
            .style()
            .set_property("display", "none")
            .expect("hide canvas");
        document
            .body()
            .expect("no body")
            .append_child(&canvas)
            .expect("append canvas");

        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .expect("get 2d context")
            .expect("no 2d context")
            .dyn_into()
            .expect("cast 2d context");

        Self {
            text_canvas: canvas,
            text_ctx_2d: ctx,
            uploaded_keys: HashSet::new(),
        }
    }
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_text_key(text: &str, font_size: f32) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    font_size.to_bits().hash(&mut hasher);
    hasher.finish()
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
        if text.is_empty() {
            return DrawCommand::Text(TextBatch {
                uploads: vec![],
                glyphs: vec![],
            });
        }

        let cache_key = hash_text_key(text, font_size);

        let font = format!("{font_size}px sans-serif");
        self.text_ctx_2d.set_font(&font);
        let metrics = self.text_ctx_2d.measure_text(text).expect("measure_text");
        let w = (metrics.width().ceil() as u32).max(1);
        let h = (font_size * 1.4).ceil() as u32;

        let mut uploads = Vec::new();
        if !self.uploaded_keys.contains(&cache_key) {
            self.text_canvas.set_width(w);
            self.text_canvas.set_height(h);

            self.text_ctx_2d.set_font(&font);
            self.text_ctx_2d.set_fill_style_str("white");
            self.text_ctx_2d.set_text_baseline("top");
            self.text_ctx_2d.clear_rect(0.0, 0.0, w as f64, h as f64);
            self.text_ctx_2d
                .fill_text(text, 0.0, (font_size * 0.15) as f64)
                .expect("fill_text");

            let image_data = self
                .text_ctx_2d
                .get_image_data(0.0, 0.0, w as f64, h as f64)
                .expect("get_image_data");
            let pixels = image_data.data();

            let rgba: Vec<u8> = pixels
                .0
                .chunks(4)
                .flat_map(|p| [255, 255, 255, p[0]])
                .collect();

            uploads.push(GlyphUpload {
                cache_key,
                width: w,
                height: h,
                data: rgba,
            });
            self.uploaded_keys.insert(cache_key);
        }

        DrawCommand::Text(TextBatch {
            uploads,
            glyphs: vec![TextGlyph {
                x,
                y,
                width: w,
                height: h,
                cache_key,
                color,
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// AppFrame
// ---------------------------------------------------------------------------

/// A frame produced by the application: draw commands plus accessibility tree.
pub struct AppFrame {
    /// Draw commands for the renderer.
    pub commands: Vec<DrawCommand>,
    /// Accessibility tree (used for DOM-based a11y on web).
    pub a11y: A11yTree,
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

async fn init_gpu_async(window: &Arc<Window>) -> GpuState {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
        ..Default::default()
    });

    let surface = instance
        .create_surface(window.clone())
        .expect("surface creation failed");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .expect("no compatible GPU adapter");

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
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

    GpuState {
        surface,
        device,
        queue,
        config,
        renderer,
    }
}

fn render_frame(gpu: &mut GpuState, commands: &[DrawCommand], viewport: [f32; 2]) {
    let frame = match gpu.surface.get_current_texture() {
        Ok(f) => f,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.surface.configure(&gpu.device, &gpu.config);
            return;
        }
        Err(_) => return,
    };

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    gpu.renderer
        .render(&gpu.device, &gpu.queue, &view, viewport, commands);
    frame.present();
}

// ---------------------------------------------------------------------------
// WebApp — ApplicationHandler for the browser
// ---------------------------------------------------------------------------

struct WebApp {
    draw_fn: Box<dyn FnMut(f32, f32, &mut TextContext, &[Event]) -> AppFrame>,
    text_ctx: Option<TextContext>,
    window: Option<Arc<Window>>,
    gpu: Rc<RefCell<Option<GpuState>>>,
    pending_events: Vec<Event>,
    cursor_pos: [f32; 2],
    modifiers: Modifiers,
    last_time: f64,
}

impl WebApp {
    fn new(draw_fn: Box<dyn FnMut(f32, f32, &mut TextContext, &[Event]) -> AppFrame>) -> Self {
        Self {
            draw_fn,
            text_ctx: None,
            window: None,
            gpu: Rc::new(RefCell::new(None)),
            pending_events: Vec::new(),
            cursor_pos: [0.0; 2],
            modifiers: Modifiers::default(),
            last_time: 0.0,
        }
    }
}

fn now_secs() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() / 1000.0)
        .unwrap_or(0.0)
}

impl ApplicationHandler for WebApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default().with_title("Sihora");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("window creation failed"),
        );

        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("no canvas");
            let document = web_sys::window()
                .expect("no window")
                .document()
                .expect("no document");
            let body = document.body().expect("no body");
            let _ = body.append_child(&canvas);

            let style = canvas.style();
            let _ = style.set_property("width", "100vw");
            let _ = style.set_property("height", "100vh");
            let _ = style.set_property("display", "block");
        }

        self.window = Some(window.clone());
        self.text_ctx = Some(TextContext::new());

        let gpu = self.gpu.clone();
        let win = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let state = init_gpu_async(&win).await;
            *gpu.borrow_mut() = Some(state);
            win.request_redraw();
        });
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
                if let Ok(mut gpu_ref) = self.gpu.try_borrow_mut() {
                    if let Some(gpu) = gpu_ref.as_mut() {
                        gpu.config.width = new_size.width.max(1);
                        gpu.config.height = new_size.height.max(1);
                        gpu.surface.configure(&gpu.device, &gpu.config);
                    }
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let has_gpu = self.gpu.try_borrow().map(|g| g.is_some()).unwrap_or(false);
                if !has_gpu {
                    return;
                }

                let (logical_w, logical_h, cfg_w, cfg_h) = {
                    let gpu_ref = self.gpu.borrow();
                    let gpu = gpu_ref.as_ref().expect("checked above");
                    match &self.window {
                        Some(w) => {
                            let s = w.scale_factor() as f32;
                            (
                                gpu.config.width as f32 / s,
                                gpu.config.height as f32 / s,
                                gpu.config.width,
                                gpu.config.height,
                            )
                        }
                        None => return,
                    }
                };

                let _ = (cfg_w, cfg_h);

                let now = now_secs();
                let dt = if self.last_time > 0.0 {
                    (now - self.last_time) as f32
                } else {
                    0.0
                };
                self.last_time = now;

                let events: Vec<Event> = std::mem::take(&mut self.pending_events);
                let text_ctx = self.text_ctx.as_mut().expect("text context present");
                let frame = (self.draw_fn)(logical_w, logical_h, text_ctx, &events);

                {
                    let mut gpu_ref = self.gpu.borrow_mut();
                    let gpu = gpu_ref.as_mut().expect("checked above");
                    render_frame(gpu, &frame.commands, [logical_w, logical_h]);
                }

                if animation::tick(dt) {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }

                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run an accessible reactive web app.
pub fn run_app(draw_fn: impl FnMut(f32, f32, &mut TextContext, &[Event]) -> AppFrame + 'static) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let event_loop = EventLoop::new().expect("event loop creation failed");
    let mut app = WebApp::new(Box::new(draw_fn));
    let _ = event_loop.run_app(&mut app);
}

/// Run a web app with a simple draw function.
pub fn run(draw_fn: impl FnMut(f32, f32) -> Vec<DrawCommand> + 'static) {
    let mut f = draw_fn;
    run_app(move |w, h, _text_ctx, _events| AppFrame {
        commands: f(w, h),
        a11y: A11yTree::empty("Sihora"),
    });
}

/// Run a web app with text support.
pub fn run_with_text(
    draw_fn: impl FnMut(f32, f32, &mut TextContext) -> Vec<DrawCommand> + 'static,
) {
    let mut f = draw_fn;
    run_app(move |w, h, text_ctx, _events| AppFrame {
        commands: f(w, h, text_ctx),
        a11y: A11yTree::empty("Sihora"),
    });
}

/// Run a reactive web app with events.
pub fn run_reactive(
    draw_fn: impl FnMut(f32, f32, &mut TextContext, &[Event]) -> Vec<DrawCommand> + 'static,
) {
    let mut f = draw_fn;
    run_app(move |w, h, text_ctx, events| AppFrame {
        commands: f(w, h, text_ctx, events),
        a11y: A11yTree::empty("Sihora"),
    });
}
