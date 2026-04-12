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
    fast: bool,
    collect_network: bool,
) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        flag.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        let info = crate::providers::collect(&config.context, fast, collect_network);
        write_frame(&info, fields, mode, &options)?;
        thread::sleep(Duration::from_secs(interval.max(1)));
    }
    if !matches!(mode, AppMode::Json) {
        writeln!(io::stdout())?;
        io::stdout().flush()?;
    }
    Ok(())
}

fn write_frame(
    info: &crate::model::MeInfo,
    fields: &[crate::model::Field],
    mode: AppMode,
    options: &RenderOptions,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    match mode {
        AppMode::Json => {
            let line = crate::output::json::render_json_line(info, fields)?;
            stdout.write_all(line.as_bytes())?;
        }
        _ => {
            let rendered = crate::app::render(info, fields, mode, options)?;
            stdout.write_all(b"\x1b[2J\x1b[H")?;
            stdout.write_all(rendered.as_bytes())?;
        }
    }
    stdout.flush()?;
    Ok(())
}
