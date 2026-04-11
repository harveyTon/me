pub mod block;
pub mod compact;
pub mod config_fmt;
pub mod json;

use crate::config::IconMode;

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub color: bool,
    pub icons: IconMode,
    pub full: bool,
}

impl RenderOptions {
    #[cfg(test)]
    pub fn plain_for_tests() -> Self {
        Self {
            color: false,
            icons: IconMode::Off,
            full: false,
        }
    }
}

pub use block::render_block;
pub use compact::render_compact;
pub use config_fmt::render_config;
pub use json::render_json;

#[cfg(test)]
mod tests;
