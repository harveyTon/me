pub fn should_color(no_color: bool) -> bool {
    !no_color && std::env::var_os("NO_COLOR").is_none() && crate::util::tty::stdout_is_tty()
}
