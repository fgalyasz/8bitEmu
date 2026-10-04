use scanline_app::{Launch, Session};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let launch = match scanline_app::parse_launch(&args) {
        Ok(launch) => launch,
        Err(error) => fail(error),
    };
    let session = match session_from(launch) {
        Ok(session) => session,
        Err(error) => fail(error),
    };
    if let Err(error) = scanline_app::run(session) {
        fail(error.to_string());
    }
}

fn session_from(launch: Launch) -> Result<Session, String> {
    let tape = choose_tape(launch.tzx, launch.tap)?;
    Ok(Session {
        model_128: launch.model_128,
        rom: read_optional(launch.rom)?,
        sna: read_optional(launch.sna)?,
        tape,
    })
}

fn choose_tape(tzx: Option<String>, tap: Option<String>) -> Result<Option<Vec<u8>>, String> {
    if let Some(path) = tzx {
        return read_optional(Some(path));
    }
    read_optional(tap)
}

fn read_optional(path: Option<String>) -> Result<Option<Vec<u8>>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) => Err(format!("{path}: {error}")),
    }
}

fn fail(message: String) -> ! {
    eprintln!("scanline: {message}");
    std::process::exit(1);
}
