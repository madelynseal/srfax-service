use crate::{common::winservice, config, Result};
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn handle_cla() -> Result<()> {
    let matches = gen_clap().get_matches();
    handle_matches(matches)?;

    Ok(())
}

fn gen_clap<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("write-config")
                .long("write-config")
                .required(false)
                .takes_value(false)
                .help("write the default config"),
        )
        .subcommand(SubCommand::with_name("run").about("run program"));

    winservice::add_to_clap(app)
}
fn handle_matches(matches: ArgMatches) -> Result<()> {
    if matches.is_present("write-config") {
        let loc = config::get_config_location();
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
