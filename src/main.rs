fn main() {
    if let Err(error) = me::app::run() {
        if me::error::is_broken_pipe(&error) {
            std::process::exit(0);
        }
        eprintln!("me: {error}");
        std::process::exit(1);
    }
}
