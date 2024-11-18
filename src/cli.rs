use crate::{common::winservice, config, Result};
use clap::{Arg, ArgMatches, Command};

pub fn handle_cla() -> Result<()> {
    let matches = gen_clap().get_matches();
    handle_matches(matches)?;

    Ok(())
}

fn gen_clap() -> Command {
    let app = command!()
        .arg(
            Arg::new("write-config")
                .long("write-config")
                .required(false)
                .num_args(0)
                .help("write the default config"),
        )
        .subcommand(Command::new("run").about("run program"));

    winservice::add_to_clap(app)
}
fn handle_matches(matches: ArgMatches) -> Result<()> {
    if matches.contains_id("write-config") {
        let loc: std::path::PathBuf = config::get_config_location();
        config::write_default_config(&loc)?;

        println!("wrote default config");
        std::process::exit(crate::EXIT_CODE_0);
    }

    let did_match: bool = if let Some(_matches) = matches.subcommand_matches("run") {
        crate::run_program()?;

        true
    } else if winservice::check_clap(&matches)? {
        true
    } else {
        false
    };

    if did_match {
        println!("done, exiting..");
        std::process::exit(crate::EXIT_CODE_0);
    }

    Ok(())
}
