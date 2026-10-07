use crate::config::AppConfig;
use crate::gpu::{GpuError, Present};
use crate::keys::{c64_key, kempston_bit, quits, spectrum_key};
use crate::launch::{session_from_launch, MachineKind, Session};
use crate::open_file::{
    self, MachineChoice, MenuBar, OpenKind, PictureSize, SaveKind, SettingsPath, MACHINE_START,
    OPEN_ID,
};
use crate::pace::{due_ticks, CATCH_UP, DISPLAY_FRAME, TURBO_SLICE};
use crate::speaker::Speaker;
use eightbit_emu_core::{
    aspect_fit, percent_size, place_percent, presented_size, BORDER, CONTENT_HEIGHT, CONTENT_WIDTH,
    Frame, Look, PresentPace, Presenter, Viewport,
};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppPhase {
    Idle,
    Running,
}

pub fn run_launcher() -> Result<(), GpuError> {
    let config = AppConfig::load();
    run_event_loop(App::launcher(config))
}

pub fn run(session: Session) -> Result<(), GpuError> {
    let model_128 = session.model_128;
    let c64 = session.machine == MachineKind::C64;
    let presenter = presenter_from(session)?;
    run_event_loop(App::running(presenter, model_128, c64))
}

fn run_event_loop(mut app: App) -> Result<(), GpuError> {
    let event_loop = EventLoop::<muda::MenuEvent>::with_user_event()
        .build()
        .map_err(show_error)?;
    let proxy = event_loop.create_proxy();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(event);
    }));
    event_loop.run_app(&mut app).map_err(show_error)
}

struct App {
    window: Option<Arc<Window>>,
    state: Option<SurfaceState>,
    presenter: Presenter,
    speaker: Speaker,
    noted: Option<String>,
    percent: Option<u32>,
    applying: bool,
    model_128: bool,
    c64: bool,
    modifiers: Modifiers,
    menu: Option<MenuBar>,
    picking: bool,
    next_tick: Instant,
    picture: Option<Frame>,
    phase: AppPhase,
    config: AppConfig,
}

struct SurfaceState {
    surface: wgpu::Surface<'static>,
    present: Present,
    config: wgpu::SurfaceConfiguration,
}

impl App {
    fn launcher(config: AppConfig) -> Self {
        Self {
            window: None,
            state: None,
            presenter: Presenter::new(PresentPace::Fixed60Hz),
            speaker: Speaker::open(),
            noted: None,
            percent: None,
            applying: false,
            model_128: config.model_128,
            c64: config.is_c64(),
            modifiers: Modifiers::default(),
            menu: None,
            picking: false,
            next_tick: Instant::now() - DISPLAY_FRAME,
            picture: None,
            phase: AppPhase::Idle,
            config,
        }
    }

    fn running(presenter: Presenter, model_128: bool, c64: bool) -> Self {
        let mut config = AppConfig::load();
        if c64 {
            config.set_c64();
        } else if model_128 {
            config.set_spectrum_128();
        } else {
            config.set_spectrum_48();
        }
        Self {
            window: None,
            state: None,
            presenter,
            speaker: Speaker::open(),
            noted: None,
            percent: None,
            applying: false,
            model_128,
            c64,
            modifiers: Modifiers::default(),
            menu: None,
            picking: false,
            next_tick: Instant::now() - DISPLAY_FRAME,
            picture: None,
            phase: AppPhase::Running,
            config,
        }
    }
}

impl ApplicationHandler<muda::MenuEvent> for App {
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
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers,
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: muda::MenuEvent) {
        let id = event.id.as_ref();
        if id == OPEN_ID {
            self.choose_file();
            return;
        }
        if id == MACHINE_START {
            self.start_machine();
            return;
        }
        if let Some(choice) = open_file::machine_choice(id) {
            self.apply_machine(choice);
            return;
        }
        if let Some(path) = open_file::settings_path(id) {
            self.pick_settings(path);
            return;
        }
        if let Some(size) = open_file::picture_size(id) {
            self.apply_picture(size);
            return;
        }
        if let Some(kind) = open_file::save_kind(id) {
            self.save_as(kind);
            return;
        }
        if id == open_file::LOADING_SOUND {
            self.apply_loading_sound();
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
        let Ok(window) = event_loop.create_window(window_attrs(self.phase, &self.presenter)) else {
            return;
        };
        let window = Arc::new(window);
        self.state = SurfaceState::new(window.clone()).ok();
        self.menu = open_file::install_menu(&window, &self.config);
        self.window = Some(window);
        if self.phase == AppPhase::Idle {
            self.start_machine();
        }
    }

    fn choose_file(&mut self) {
        if self.picking {
            return;
        }
        self.picking = true;
        let path = open_file::pick_path(self.window.as_deref());
        self.picking = false;
        if let Some(path) = path {
            self.open_user_path(&path);
        }
    }

    fn apply_machine(&mut self, choice: MachineChoice) {
        match choice {
            MachineChoice::Spectrum48 => self.config.set_spectrum_48(),
            MachineChoice::Spectrum128 => self.config.set_spectrum_128(),
            MachineChoice::C64 => self.config.set_c64(),
        }
        self.persist_config();
        if let Some(menu) = &self.menu {
            menu.sync_machine(&self.config);
        }
        self.start_machine();
    }

    fn pick_settings(&mut self, kind: SettingsPath) {
        if self.picking {
            return;
        }
        self.picking = true;
        let path = open_file::pick_rom_path(self.window.as_deref(), kind);
        self.picking = false;
        let Some(path) = path else {
            return;
        };
        self.apply_settings_path(kind, path.to_string_lossy().into_owned());
    }

    fn apply_settings_path(&mut self, kind: SettingsPath, path: String) {
        match kind {
            SettingsPath::RomFolder => self.config.rom_folder = Some(path),
            SettingsPath::SpectrumRom => self.config.spectrum_rom = Some(path),
            SettingsPath::Kernal => self.config.kernal = Some(path),
            SettingsPath::Basic => self.config.basic = Some(path),
            SettingsPath::Chargen => self.config.chargen = Some(path),
        }
        self.persist_config();
    }

    fn persist_config(&mut self) {
        if let Err(error) = self.config.save() {
            eprintln!("8bitemu: {error}");
        }
    }

    fn start_machine(&mut self) {
        let launch = match self.config.to_launch() {
            Ok(launch) => launch,
            Err(error) => {
                eprintln!("8bitemu: {error}");
                return;
            }
        };
        let session = match session_from_launch(launch) {
            Ok(session) => session,
            Err(error) => {
                eprintln!("8bitemu: {error}");
                return;
            }
        };
        let model_128 = session.model_128;
        let c64 = session.machine == MachineKind::C64;
        let presenter = match presenter_from(session) {
            Ok(presenter) => presenter,
            Err(error) => {
                eprintln!("8bitemu: {error}");
                return;
            }
        };
        self.presenter = presenter;
        self.model_128 = model_128;
        self.c64 = c64;
        self.phase = AppPhase::Running;
        self.picture = None;
        self.next_tick = Instant::now() - DISPLAY_FRAME;
        if let Some(menu) = &self.menu {
            menu.sync_machine(&self.config);
        }
        self.retitle();
    }

    fn open_user_path(&mut self, path: &std::path::Path) {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("8bitemu: {}: {error}", path.display());
                return;
            }
        };
        let name = path.to_string_lossy();
        let Some(kind) = open_file::open_kind(&name) else {
            eprintln!("8bitemu: {name} is not a rom, sna, tap, tzx, or prg");
            return;
        };
        self.prepare_machine_for(kind, path, bytes.len());
        if self.phase != AppPhase::Running {
            return;
        }
        if let Err(error) = self.apply_open(kind, &bytes) {
            eprintln!("8bitemu: {error}");
        }
    }

    fn prepare_machine_for(&mut self, kind: OpenKind, path: &std::path::Path, len: usize) {
        self.config.apply_open_kind(kind, len);
        if kind == OpenKind::Rom {
            self.config.spectrum_rom = Some(path.to_string_lossy().into_owned());
        }
        self.persist_config();
        let needs_boot = self.phase != AppPhase::Running || machine_mismatch(self, kind);
        if needs_boot {
            self.start_machine();
        }
    }

    fn apply_open(&mut self, kind: OpenKind, bytes: &[u8]) -> Result<(), eightbit_emu_core::CoreError> {
        match kind {
            OpenKind::Rom => self.open_rom(bytes),
            OpenKind::Sna => self.open_sna(bytes),
            OpenKind::Tape => self.open_tape(bytes),
            OpenKind::Prg => self.open_prg(bytes),
        }
    }

    fn open_prg(&mut self, bytes: &[u8]) -> Result<(), eightbit_emu_core::CoreError> {
        self.presenter.warm_c64(250)?;
        self.presenter.load_prg(bytes)
    }

    fn open_rom(&mut self, bytes: &[u8]) -> Result<(), eightbit_emu_core::CoreError> {
        let model = open_file::rom_model_128(bytes.len(), self.model_128);
        self.presenter.load_rom(bytes, model)?;
        self.model_128 = model;
        self.config.model_128 = model;
        Ok(())
    }

    fn open_sna(&mut self, bytes: &[u8]) -> Result<(), eightbit_emu_core::CoreError> {
        self.presenter.load_sna(bytes)?;
        self.model_128 = open_file::sna_model_128(bytes.len(), self.model_128);
        self.config.model_128 = self.model_128;
        Ok(())
    }

    fn open_tape(&mut self, bytes: &[u8]) -> Result<(), eightbit_emu_core::CoreError> {
        let booted = self.presenter.is_booted();
        self.presenter.load_tape(bytes)?;
        if booted {
            self.presenter.reset();
        }
        Ok(())
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
        if event.repeat {
            return;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let down = event.state == ElementState::Pressed;
        if down && quits(code) {
            event_loop.exit();
            return;
        }
        if down && self.start_shortcut(code) {
            return;
        }
        if down && open_file::command_open(self.modifiers.state(), code) {
            self.choose_file();
            return;
        }
        if self.phase != AppPhase::Running {
            return;
        }
        if down && self.command_save(code) {
            return;
        }
        if down && self.select_look(code) {
            return;
        }
        if down && self.loading_sound_key(code) {
            return;
        }
        if down && code == KeyCode::F12 {
            self.presenter.reset();
            return;
        }
        if down && self.scale_key(code) {
            return;
        }
        if down && code == KeyCode::F9 {
            self.presenter.resume_tape();
            return;
        }
        if down && code == KeyCode::F11 {
            write_named("8bitemu.sna", self.presenter.snapshot());
            return;
        }
        if !self.c64 && stick_key(self, code, down) {
            return;
        }
        hold_key_target(self, code, down);
    }

    fn start_shortcut(&mut self, code: KeyCode) -> bool {
        if !open_file::command_start(self.modifiers.state(), code) {
            return false;
        }
        self.start_machine();
        true
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
        let Some(window) = &self.window else {
            return;
        };
        if self.phase == AppPhase::Idle {
            window.set_title("8bitEmu");
            return;
        }
        window.set_title(self.presenter.look().title());
    }

    fn resize(&mut self, width: u32, height: u32) {
        if self.applying {
            self.applying = false;
        } else {
            self.percent = None;
        }
        if let Some(state) = &mut self.state {
            state.resize(width, height);
        }
    }

    fn command_save(&mut self, code: KeyCode) -> bool {
        let Some(kind) = open_file::command_save(self.modifiers.state(), code) else {
            return false;
        };
        self.save_as(kind);
        true
    }

    fn apply_picture(&mut self, size: PictureSize) {
        self.percent = size.percent();
        self.applying = true;
        let (width, height) = picture_window(size.percent());
        self.request_inner(width, height);
    }

    fn request_inner(&self, width: u32, height: u32) {
        let Some(window) = &self.window else {
            return;
        };
        let size = LogicalSize::new(f64::from(width), f64::from(height));
        let _ = window.request_inner_size(size);
    }

    fn save_as(&mut self, kind: SaveKind) {
        if self.phase != AppPhase::Running || self.picking {
            return;
        }
        match prepared_save(&self.presenter, kind) {
            Ok(Some(bytes)) => self.ask_and_write(kind, &bytes),
            Ok(None) => eprintln!("8bitemu: no recording to save"),
            Err(error) => eprintln!("8bitemu: {error}"),
        }
    }

    fn ask_and_write(&mut self, kind: SaveKind, bytes: &[u8]) {
        self.picking = true;
        let path = open_file::pick_save(self.window.as_deref(), kind);
        self.picking = false;
        let Some(path) = path else {
            return;
        };
        let named = open_file::ensure_extension(&path.to_string_lossy(), kind.extension());
        write_bytes(&named, bytes);
    }

    fn scale_key(&mut self, code: KeyCode) -> bool {
        let Some(percent) = percent_from(code) else {
            return false;
        };
        self.apply_picture(PictureSize::Percent(percent));
        true
    }

    fn apply_loading_sound(&mut self) {
        let Some(bar) = &self.menu else {
            return;
        };
        self.presenter.set_loading_sound(bar.loading_sound.is_checked());
    }

    fn loading_sound_key(&mut self, code: KeyCode) -> bool {
        if code != KeyCode::F4 || menu_toggles_sound() {
            return false;
        }
        let on = !self.presenter.loading_sound();
        self.presenter.set_loading_sound(on);
        true
    }

    fn redraw(&mut self) {
        if self.phase == AppPhase::Idle {
            self.clear_idle();
            return;
        }
        self.catch_up();
        let Some(frame) = self.picture.clone() else {
            return;
        };
        let look = self.presenter.look();
        let message = self.paint_message(&frame, look, self.percent);
        self.note_message(message);
    }

    fn clear_idle(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        if let Err(error) = state.clear() {
            self.note_message(Some(error.to_string()));
        }
    }

    fn catch_up(&mut self) {
        let now = Instant::now();
        let late = now.saturating_duration_since(self.next_tick);
        let ticks = due_ticks(late, DISPLAY_FRAME, CATCH_UP);
        let mut ran = 0u32;
        while ran < ticks {
            self.play_tick();
            self.next_tick += DISPLAY_FRAME;
            ran += 1;
        }
        if ticks == CATCH_UP {
            self.next_tick = now;
        }
    }

    fn play_tick(&mut self) {
        let frame = self.presenter.on_display_tick().ok().cloned();
        let Some(frame) = frame else {
            return;
        };
        self.picture = Some(frame);
        let audio = self.presenter.take_audio();
        if self.presenter.hears_loading() {
            self.speaker.push(&audio);
        }
        if let Some(bytes) = self.presenter.take_tap() {
            write_bytes("8bitemu.tap", &bytes);
        }
        self.rush_load();
    }

    fn rush_load(&mut self) {
        let start = Instant::now();
        let mut moved = false;
        while start.elapsed() < TURBO_SLICE {
            let Ok(true) = self.presenter.rush_cpu() else {
                break;
            };
            moved = true;
        }
        if !moved {
            return;
        }
        if let Ok(frame) = self.presenter.paint_latest() {
            self.picture = Some(frame);
        }
    }

    fn paint_message(&mut self, frame: &Frame, look: Look, percent: Option<u32>) -> Option<String> {
        let state = self.state.as_mut()?;
        state.paint(frame, look, percent).err().map(|error| error.to_string())
    }

    fn note_message(&mut self, message: Option<String>) {
        let Some(message) = message else {
            self.noted = None;
            return;
        };
        if self.noted.as_deref() == Some(message.as_str()) {
            return;
        }
        eprintln!("8bitemu: {message}");
        self.noted = Some(message);
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

    fn paint(&mut self, frame: &Frame, look: Look, percent: Option<u32>) -> Result<(), GpuError> {
        let surface_texture = current_texture(&self.surface)?;
        let view = picture_view(frame, self.config.width, self.config.height, percent);
        self.present.draw_to(
            &surface_texture.texture,
            frame,
            look,
            view.width,
            view.height,
            view.x,
            view.y,
        )?;
        self.present.present_surface(surface_texture);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), GpuError> {
        let surface_texture = current_texture(&self.surface)?;
        self.present.clear_surface(surface_texture)
    }
}

fn present_device(present: &Present) -> &wgpu::Device {
    present.device()
}

fn picture_view(frame: &Frame, width: u32, height: u32, percent: Option<u32>) -> Viewport {
    let src_w = u32::from(frame.width);
    let src_h = u32::from(frame.height);
    if let Some(percent) = percent {
        return place_percent(src_w, src_h, width, height, percent);
    }
    aspect_fit(src_w, src_h, width, height)
}

fn window_attrs(phase: AppPhase, presenter: &Presenter) -> winit::window::WindowAttributes {
    let title = if phase == AppPhase::Idle {
        "8bitEmu"
    } else {
        presenter.look().title()
    };
    winit::window::Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(960.0, 720.0))
}

fn machine_mismatch(app: &App, kind: OpenKind) -> bool {
    match kind {
        OpenKind::Prg => !app.c64,
        OpenKind::Rom | OpenKind::Sna | OpenKind::Tape => app.c64,
    }
}

fn menu_toggles_sound() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
}

fn picture_window(percent: Option<u32>) -> (u32, u32) {
    let Some(percent) = percent else {
        return (960, 720);
    };
    let (width, height) = presented_size(CONTENT_WIDTH, CONTENT_HEIGHT, BORDER);
    percent_size(u32::from(width), u32::from(height), percent)
}

fn prepared_save(
    presenter: &eightbit_emu_core::Presenter,
    kind: SaveKind,
) -> Result<Option<Vec<u8>>, eightbit_emu_core::CoreError> {
    match kind {
        SaveKind::Sna => presenter.snapshot().map(Some),
        SaveKind::Tap => Ok(presenter.tap_bytes()),
        SaveKind::Tzx => Ok(presenter.tzx_bytes()),
    }
}

fn hold_key_target(app: &mut App, code: KeyCode, down: bool) {
    if app.c64 {
        let Some((row, col)) = c64_key(code) else {
            return;
        };
        app.presenter.set_c64_key(row, col, down);
        return;
    }
    let Some((row, mask)) = spectrum_key(code) else {
        return;
    };
    app.presenter.set_key(row, mask, down);
}

fn stick_key(app: &mut App, code: KeyCode, down: bool) -> bool {
    let Some(mask) = kempston_bit(code) else {
        return false;
    };
    app.presenter.set_stick(mask, down);
    true
}

fn percent_from(code: KeyCode) -> Option<u32> {
    match code {
        KeyCode::F5 => Some(125),
        KeyCode::F6 => Some(150),
        KeyCode::F7 => Some(175),
        KeyCode::F8 => Some(200),
        _ => None,
    }
}

fn write_named(name: &str, image: Result<Vec<u8>, impl std::fmt::Display>) {
    match image {
        Ok(bytes) => write_bytes(name, &bytes),
        Err(error) => eprintln!("8bitemu: {error}"),
    }
}

fn write_bytes(name: &str, bytes: &[u8]) {
    if let Err(error) = std::fs::write(name, bytes) {
        eprintln!("8bitemu: {name}: {error}");
    }
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
    if session.machine == MachineKind::C64 {
        return presenter_c64(presenter, session);
    }
    if let Some(rom) = session.rom {
        presenter
            .load_rom(&rom, session.model_128)
            .map_err(show_error)?;
    }
    if let Some(tape) = session.tape {
        presenter.load_tape(&tape).map_err(show_error)?;
    }
    if let Some(sna) = session.sna {
        presenter.load_sna(&sna).map_err(show_error)?;
    }
    Ok(presenter)
}

fn presenter_c64(mut presenter: Presenter, session: Session) -> Result<Presenter, GpuError> {
    boot_c64_roms(&mut presenter, &session)?;
    boot_c64_prg(&mut presenter, session.prg)?;
    Ok(presenter)
}

fn boot_c64_roms(presenter: &mut Presenter, session: &Session) -> Result<(), GpuError> {
    let kernal = session.kernal.as_ref().ok_or_else(|| missing_rom("kernal"))?;
    let basic = session.basic.as_ref().ok_or_else(|| missing_rom("basic"))?;
    let chargen = session
        .chargen
        .as_ref()
        .ok_or_else(|| missing_rom("chargen"))?;
    presenter
        .load_c64_roms(kernal, basic, chargen)
        .map_err(show_error)
}

fn boot_c64_prg(presenter: &mut Presenter, prg: Option<Vec<u8>>) -> Result<(), GpuError> {
    let Some(bytes) = prg else {
        return Ok(());
    };
    presenter.warm_c64(250).map_err(show_error)?;
    presenter.load_prg(&bytes).map_err(show_error)
}

fn missing_rom(name: &str) -> GpuError {
    GpuError::new(format!("missing C64 {name} ROM"))
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
