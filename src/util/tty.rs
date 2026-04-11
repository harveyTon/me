use is_terminal::IsTerminal;

pub fn stdout_is_tty() -> bool {
    std::io::stdout().is_terminal()
}
