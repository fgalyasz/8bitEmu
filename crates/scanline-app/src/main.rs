fn main() {
    if let Err(error) = scanline_app::run() {
        eprintln!("scanline: {error}");
        std::process::exit(1);
    }
}
