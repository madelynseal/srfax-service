extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate base64;
extern crate flexi_logger;
extern crate log_panics;
extern crate reqwest;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate unwrap;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

#[cfg(windows)]
#[macro_use]
extern crate windows_service;

mod cli;
mod common;
mod config;
mod email;
mod response;
mod srfax;
mod srfax_service;

#[cfg(windows)]
mod main_ws;

pub use anyhow::Result;
use config::CONFIG;

use std::time;

pub const EXIT_CODE_0: i32 = 0;

#[cfg(windows)]
const SERVICE_NAME: &str = "SRFax";
const EMAIL_SUBJECT_PREFIX: &str = "SRFax Service: ";

#[cfg(not(windows))]
fn main() -> Result<()> {
    common::set_cwd_to_exe()?;
    config::check_config_exists()?;

    cli::handle_cla()?;

    run_program()?;

    Ok(())
}
#[cfg(windows)]
fn main() -> Result<()> {
    common::set_cwd_to_exe()?;
    config::check_config_exists()?;

    cli::handle_cla()?;

    main_ws::main()?;

    Ok(())
}

pub fn run_program() -> Result<()> {
    let tick_time = time::Duration::from_secs(CONFIG.tick_rate);

    setup_logging()?;

    // start service
    srfax_service::run_srfax_service(tick_time);

    info!("done");
    Ok(())
}

fn setup_logging() -> Result<()> {
    use flexi_logger::{opt_format, Duplicate, FileSpec, Logger};

    let log_dir = if let Some(ref dir) = CONFIG.log.dir {
        dir.to_string()
    } else {
        String::from("logs")
    };

    let mut log = Logger::try_with_str(&CONFIG.log.level)?
        .log_to_file(FileSpec::default().directory(log_dir))
        .format(opt_format);

    if CONFIG.log.stdout {
        log = log.duplicate_to_stderr(Duplicate::All);
    }
    if cfg!(windows) {
        log = log.use_windows_line_ending();
    }

    let _log_handle = log.start()?;
    // log panics to log
    log_panics::init();

    print_info();

    Ok(())
}

fn print_info() {
    info!("VERSION: {}", env!("CARGO_PKG_VERSION"));
    info!("VERSION: COMMIT {}", env!("VERGEN_GIT_SHA"));
    info!(
        "VERSION: BUILD TIMESTAMP {}",
        env!("VERGEN_BUILD_TIMESTAMP")
    );
    info!("VERSION: TARGET {}", env!("VERGEN_RUSTC_HOST_TRIPLE"));
}
