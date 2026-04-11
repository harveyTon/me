fn main() {
    if let Err(error) = me::app::run() {
        eprintln!("me: {error}");
        std::process::exit(1);
    }
}
