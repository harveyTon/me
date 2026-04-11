use crate::config::IconMode;

pub fn shell_prefix(mode: IconMode) -> &'static str {
    match mode {
        IconMode::Auto | IconMode::On | IconMode::Off => "",
    }
}

pub fn field_icon(field: &str) -> &'static str {
    let _ = field;
    ""
}
