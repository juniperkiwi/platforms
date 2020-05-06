fn main() {
    amethyst::start_logger(Default::default());

    match platforms::run() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
