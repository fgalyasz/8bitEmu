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
    pub kernal: Option<String>,
    pub basic: Option<String>,
    pub chargen: Option<String>,
}

pub struct Session {
    pub machine: MachineKind,
    pub model_128: bool,
    pub rom: Option<Vec<u8>>,
    pub sna: Option<Vec<u8>>,
    pub tape: Option<Vec<u8>>,
    pub kernal: Option<Vec<u8>>,
    pub basic: Option<Vec<u8>>,
    pub chargen: Option<Vec<u8>>,
}

pub fn parse_launch(args: &[String]) -> Result<Launch, String> {
    let mut launch = Launch {
        machine: MachineKind::Spectrum,
        model_128: false,
        rom: None,
        sna: None,
        tap: None,
        tzx: None,
        kernal: None,
        basic: None,
        chargen: None,
    };
    let mut index = 0;
    while index < args.len() {
        index = take_arg(&mut launch, args, index)?;
    }
    apply_c64_defaults(&mut launch);
    Ok(launch)
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
