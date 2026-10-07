#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineKind {
    Spectrum,
    C64,
}

#[derive(Debug)]
pub struct Launch {
    pub machine: MachineKind,
    pub model_128: bool,
    pub rom: Option<String>,
    pub sna: Option<String>,
    pub tap: Option<String>,
    pub tzx: Option<String>,
    pub prg: Option<String>,
    pub kernal: Option<String>,
    pub basic: Option<String>,
    pub chargen: Option<String>,
}

#[derive(Debug)]
pub enum LaunchMode {
    Launcher,
    Boot(Launch),
}

pub struct Session {
    pub machine: MachineKind,
    pub model_128: bool,
    pub rom: Option<Vec<u8>>,
    pub sna: Option<Vec<u8>>,
    pub tape: Option<Vec<u8>>,
    pub prg: Option<Vec<u8>>,
    pub kernal: Option<Vec<u8>>,
    pub basic: Option<Vec<u8>>,
    pub chargen: Option<Vec<u8>>,
}

pub fn parse_mode(args: &[String]) -> Result<LaunchMode, String> {
    if wants_boot(args) {
        return Ok(LaunchMode::Boot(parse_launch(args)?));
    }
    if let Some(error) = launcher_arg_error(args) {
        return Err(error);
    }
    Ok(LaunchMode::Launcher)
}

pub fn wants_boot(args: &[String]) -> bool {
    args.iter().any(|arg| is_boot_flag(arg))
}

pub fn session_from_launch(launch: Launch) -> Result<Session, String> {
    let tape = choose_tape(launch.tzx, launch.tap)?;
    Ok(Session {
        machine: launch.machine,
        model_128: launch.model_128,
        rom: read_optional(launch.rom)?,
        sna: read_optional(launch.sna)?,
        tape,
        prg: read_optional(launch.prg)?,
        kernal: read_optional(launch.kernal)?,
        basic: read_optional(launch.basic)?,
        chargen: read_optional(launch.chargen)?,
    })
}

pub fn parse_launch(args: &[String]) -> Result<Launch, String> {
    let mut launch = Launch {
        machine: MachineKind::Spectrum,
        model_128: false,
        rom: None,
        sna: None,
        tap: None,
        tzx: None,
        prg: None,
        kernal: None,
        basic: None,
        chargen: None,
    };
    let mut index = 0;
    while index < args.len() {
        index = take_arg(&mut launch, args, index)?;
    }
    apply_prg_machine(&mut launch);
    apply_c64_defaults(&mut launch);
    Ok(launch)
}

fn is_boot_flag(arg: &str) -> bool {
    matches!(
        arg,
        "--machine"
            | "--rom"
            | "--sna"
            | "--tap"
            | "--tzx"
            | "--prg"
            | "--kernal"
            | "--basic"
            | "--chargen"
    )
}

fn launcher_arg_error(args: &[String]) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--model" => match next_value(args, index, "--model").and_then(model_flag) {
                Ok(_) => index += 2,
                Err(error) => return Some(error),
            },
            other => return Some(format!("unknown argument {other}")),
        }
    }
    None
}

fn apply_prg_machine(launch: &mut Launch) {
    if launch.prg.is_some() {
        launch.machine = MachineKind::C64;
    }
}

fn apply_c64_defaults(launch: &mut Launch) {
    if launch.machine != MachineKind::C64 {
        return;
    }
    if launch.kernal.is_none() {
        launch.kernal = Some("roms/c64-kernal.rom".to_string());
    }
    if launch.basic.is_none() {
        launch.basic = Some("roms/c64-basic.rom".to_string());
    }
    if launch.chargen.is_none() {
        launch.chargen = Some("roms/c64-chargen.rom".to_string());
    }
}

fn take_arg(launch: &mut Launch, args: &[String], index: usize) -> Result<usize, String> {
    match args[index].as_str() {
        "--machine" => take_machine(launch, args, index),
        "--model" => take_model(launch, args, index),
        "--rom" => take_path(&mut launch.rom, args, index, "--rom"),
        "--sna" => take_path(&mut launch.sna, args, index, "--sna"),
        "--tap" => take_path(&mut launch.tap, args, index, "--tap"),
        "--tzx" => take_path(&mut launch.tzx, args, index, "--tzx"),
        "--prg" => take_path(&mut launch.prg, args, index, "--prg"),
        "--kernal" => take_path(&mut launch.kernal, args, index, "--kernal"),
        "--basic" => take_path(&mut launch.basic, args, index, "--basic"),
        "--chargen" => take_path(&mut launch.chargen, args, index, "--chargen"),
        other => Err(format!("unknown argument {other}")),
    }
}

fn take_machine(launch: &mut Launch, args: &[String], index: usize) -> Result<usize, String> {
    let value = next_value(args, index, "--machine")?;
    launch.machine = machine_kind(value)?;
    Ok(index + 2)
}

fn machine_kind(value: &str) -> Result<MachineKind, String> {
    match value {
        "spectrum" => Ok(MachineKind::Spectrum),
        "c64" => Ok(MachineKind::C64),
        _ => Err(format!("unknown machine {value}")),
    }
}

fn take_model(launch: &mut Launch, args: &[String], index: usize) -> Result<usize, String> {
    let value = next_value(args, index, "--model")?;
    launch.model_128 = model_flag(value)?;
    Ok(index + 2)
}

fn model_flag(value: &str) -> Result<bool, String> {
    match value {
        "48" => Ok(false),
        "128" => Ok(true),
        _ => Err(format!("unknown model {value}")),
    }
}

fn take_path(
    slot: &mut Option<String>,
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<usize, String> {
    *slot = Some(next_value(args, index, flag)?.to_string());
    Ok(index + 2)
}

fn next_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
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
