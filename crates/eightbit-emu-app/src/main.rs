use eightbit_emu_app::LaunchMode;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = match eightbit_emu_app::parse_mode(&args) {
        Ok(mode) => mode,
        Err(error) => fail(error),
    };
    match mode {
        LaunchMode::Launcher => run_launcher(),
        LaunchMode::Boot(launch) => run_boot(launch),
    }
}

fn run_launcher() {
    if let Err(error) = eightbit_emu_app::run_launcher() {
        fail(error.to_string());
    }
}

fn run_boot(launch: eightbit_emu_app::Launch) {
    let session = match eightbit_emu_app::session_from_launch(launch) {
        Ok(session) => session,
        Err(error) => fail(error),
    };
    if let Err(error) = eightbit_emu_app::run(session) {
        fail(error.to_string());
    }
}

fn fail(message: String) -> ! {
    eprintln!("8bitemu: {message}");
    std::process::exit(1);
}
