use crate::config::ColorMode;

pub fn should_color(no_color: bool, mode: ColorMode) -> bool {
    resolve_color(
        no_color,
        std::env::var_os("NO_COLOR").is_some(),
        crate::util::tty::stdout_is_tty(),
        mode,
    )
}

fn resolve_color(no_color: bool, no_color_env: bool, stdout_is_tty: bool, mode: ColorMode) -> bool {
    if no_color || no_color_env {
        return false;
    }
    match mode {
        ColorMode::Auto => stdout_is_tty,
        ColorMode::On => true,
        ColorMode::Off => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_enables_color_for_tty() {
        assert!(resolve_color(false, false, true, ColorMode::Auto));
        assert!(!resolve_color(false, false, false, ColorMode::Auto));
    }

    #[test]
    fn on_forces_color_when_no_overrides_exist() {
        assert!(resolve_color(false, false, true, ColorMode::On));
        assert!(resolve_color(false, false, false, ColorMode::On));
    }

    #[test]
    fn off_disables_color() {
        assert!(!resolve_color(false, false, true, ColorMode::Off));
    }

    #[test]
    fn no_color_overrides_configured_on_mode() {
        assert!(!resolve_color(true, false, true, ColorMode::On));
        assert!(!resolve_color(false, true, true, ColorMode::On));
    }
}
