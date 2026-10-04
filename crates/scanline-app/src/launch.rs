#[derive(Debug)]
pub struct Launch {
    pub model_128: bool,
    pub rom: Option<String>,
    pub sna: Option<String>,
    pub tap: Option<String>,
    pub tzx: Option<String>,
}

pub struct Session {
    pub model_128: bool,
    pub rom: Option<Vec<u8>>,
    pub sna: Option<Vec<u8>>,
    pub tape: Option<Vec<u8>>,
}

pub fn parse_launch(args: &[String]) -> Result<Launch, String> {
    let mut launch = Launch {
        model_128: false,
        rom: None,
        sna: None,
        tap: None,
        tzx: None,
    };
    let mut index = 0;
    while index < args.len() {
        index = take_arg(&mut launch, args, index)?;
    }
    Ok(launch)
}

fn take_arg(launch: &mut Launch, args: &[String], index: usize) -> Result<usize, String> {
    match args[index].as_str() {
        "--model" => take_model(launch, args, index),
        "--rom" => take_path(&mut launch.rom, args, index, "--rom"),
        "--sna" => take_path(&mut launch.sna, args, index, "--sna"),
        "--tap" => take_path(&mut launch.tap, args, index, "--tap"),
        "--tzx" => take_path(&mut launch.tzx, args, index, "--tzx"),
        other => Err(format!("unknown argument {other}")),
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
