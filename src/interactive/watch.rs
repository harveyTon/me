use crate::{app::AppMode, config::Config, output::RenderOptions};
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub fn run(
    config: &Config,
    fields: &[crate::model::Field],
    mode: AppMode,
    options: RenderOptions,
    interval: u64,
) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        flag.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        let info = crate::providers::collect(&config.context);
        print!(
            "\x1b[2J\x1b[H{}",
            crate::app::render(&info, fields, mode, &options)?
        );
        io::stdout().flush()?;
        thread::sleep(Duration::from_secs(interval.max(1)));
    }
    println!();
    Ok(())
}
