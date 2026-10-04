use crate::gpu::{GpuError, Present, fit_scale};
use crate::keys::spectrum_key;
use crate::launch::Session;
use crate::speaker::Speaker;
use scanline_core::{Frame, Look, PresentPace, Presenter};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub fn run(session: Session) -> Result<(), GpuError> {
    let presenter = presenter_from(session)?;
    let event_loop = EventLoop::new().map_err(show_error)?;
    let mut app = App::new(presenter);
    event_loop.run_app(&mut app).map_err(show_error)
}

struct App {
    window: Option<Arc<Window>>,
    state: Option<SurfaceState>,
    presenter: Presenter,
    speaker: Speaker,
    noted: Option<String>,
}

struct SurfaceState {
    surface: wgpu::Surface<'static>,
    present: Present,
    config: wgpu::SurfaceConfiguration,
}

impl App {
    fn new(presenter: Presenter) -> Self {
        Self {
            window: None,
            state: None,
            presenter,
            speaker: Speaker::open(),
            noted: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        self.open(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.key(event_loop, event),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    fn open(&mut self, event_loop: &ActiveEventLoop) {
        let Ok(window) = event_loop.create_window(window_attrs(&self.presenter)) else {
            return;
        };
        let window = Arc::new(window);
        self.state = SurfaceState::new(window.clone()).ok();
        self.window = Some(window);
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
        if event.repeat {
            return;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let down = event.state == ElementState::Pressed;
        if down && code == KeyCode::Escape {
            event_loop.exit();
            return;
        }
        if down && self.select_look(code) {
            return;
        }
        if down && code == KeyCode::F12 {
            self.presenter.reset();
            return;
        }
        hold_key_target(self, code, down);
    }

    fn select_look(&mut self, code: KeyCode) -> bool {
        let Some(look) = look_from_code(code) else {
            return false;
        };
        self.presenter.set_look(look);
        self.retitle();
        true
    }

    fn retitle(&self) {
        if let Some(window) = &self.window {
            window.set_title(self.presenter.look().title());
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Some(state) = &mut self.state {
            state.resize(width, height);
        }
    }

    fn redraw(&mut self) {
        let frame = self.next_frame();
        self.speaker.push(&self.presenter.take_audio());
        let Some(frame) = frame else {
            return;
        };
        let look = self.presenter.look();
        let message = self.paint_message(&frame, look);
        self.note_message(message);
    }

    fn paint_message(&mut self, frame: &Frame, look: Look) -> Option<String> {
        let state = self.state.as_mut()?;
        state.paint(frame, look).err().map(|error| error.to_string())
    }

    fn note_message(&mut self, message: Option<String>) {
        let Some(message) = message else {
            self.noted = None;
            return;
        };
        if self.noted.as_deref() == Some(message.as_str()) {
            return;
        }
        eprintln!("scanline: {message}");
        self.noted = Some(message);
    }

    fn next_frame(&mut self) -> Option<Frame> {
        self.presenter.on_display_tick().ok().cloned()
    }
}

impl SurfaceState {
    fn new(window: Arc<Window>) -> Result<Self, GpuError> {
        let (present, surface) =
            Present::with_surface(window.clone(), wgpu::TextureFormat::Bgra8Unorm)?;
        let size = window.inner_size();
        let config = surface_config(size.width.max(1), size.height.max(1));
        surface.configure(&present_device(&present), &config);
        Ok(Self {
            surface,
            present,
            config,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface
            .configure(present_device(&self.present), &self.config);
    }

    fn paint(&mut self, frame: &Frame, look: Look) -> Result<(), GpuError> {
        let surface_texture = current_texture(&self.surface)?;
        let (scale, origin_x, origin_y) = fit(frame, self.config.width, self.config.height);
        self.present.draw_to(
            &surface_texture.texture,
            frame,
            look,
            scale,
            origin_x,
            origin_y,
        )?;
        self.present.present_surface(surface_texture);
        Ok(())
    }
}

fn present_device(present: &Present) -> &wgpu::Device {
    present.device()
}

fn fit(frame: &Frame, width: u32, height: u32) -> (u32, u32, u32) {
    fit_scale(
        u32::from(frame.width),
        u32::from(frame.height),
        width,
        height,
    )
    .unwrap_or((1, 0, 0))
}

fn window_attrs(presenter: &Presenter) -> winit::window::WindowAttributes {
    winit::window::Window::default_attributes()
        .with_title(presenter.look().title())
        .with_inner_size(LogicalSize::new(960.0, 720.0))
}

fn hold_key_target(app: &mut App, code: KeyCode, down: bool) {
    let Some((row, mask)) = spectrum_key(code) else {
        return;
    };
    app.presenter.set_key(row, mask, down);
}

fn look_from_code(code: KeyCode) -> Option<Look> {
    match code {
        KeyCode::F1 => Look::from_digit(1),
        KeyCode::F2 => Look::from_digit(2),
        KeyCode::F3 => Look::from_digit(3),
        _ => None,
    }
}

fn presenter_from(session: Session) -> Result<Presenter, GpuError> {
    let mut presenter = Presenter::new(PresentPace::Fixed60Hz);
    if let Some(rom) = session.rom {
        presenter
            .load_rom(&rom, session.model_128)
            .map_err(show_error)?;
    }
    if let Some(sna) = session.sna {
        presenter.load_sna(&sna).map_err(show_error)?;
    }
    Ok(presenter)
}

fn surface_config(width: u32, height: u32) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
        color_space: wgpu::SurfaceColorSpace::Auto,
    }
}

fn current_texture(surface: &wgpu::Surface) -> Result<wgpu::SurfaceTexture, GpuError> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(texture) => Ok(texture),
        wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(texture),
        wgpu::CurrentSurfaceTexture::Timeout => Err(GpuError::new("surface timeout".to_string())),
        wgpu::CurrentSurfaceTexture::Occluded => Err(GpuError::new("surface occluded".to_string())),
        wgpu::CurrentSurfaceTexture::Outdated => Err(GpuError::new("surface outdated".to_string())),
        wgpu::CurrentSurfaceTexture::Lost => Err(GpuError::new("surface lost".to_string())),
        wgpu::CurrentSurfaceTexture::Validation => {
            Err(GpuError::new("surface validation".to_string()))
        }
    }
}

fn show_error(error: impl std::fmt::Display) -> GpuError {
    GpuError::new(error.to_string())
}
