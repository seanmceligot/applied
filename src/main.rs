
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dotenv;
#[macro_use]
extern crate clap;
//#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate config;
extern crate dirs;

use config::Config;
use failure::Error;
use clap::{App, Arg};
use std::{io::{self, Write}, path::PathBuf, process::Command};
use ansi_term::Colour::{Green, Red, Yellow};
mod applyerr;
use applyerr::ApplyError;
mod action;
use action::Action;

fn arguments<'a>() -> clap::ArgMatches<'a> {
    return  App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .help(crate_description!())
        .arg(Arg::with_name("name")
            .short("n")
            .long("name")
            .value_name("NAME")
			.required(true)
            .help("name")
            .takes_value(true))
		.arg(Arg::with_name("unapply")
			.short("u")
			.long("unapply")
			.conflicts_with("apply")
			.conflicts_with("show")
			)
		.arg(Arg::with_name("show")
			.short("s")
			.long("show")
			.conflicts_with("apply")
			.conflicts_with("unapply")
			)
		.arg(Arg::with_name("apply")
			.short("a")
			.long("apply")
			.conflicts_with("unapply")
			.conflicts_with("show")
			)
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .takes_value(true)
            .help("verbosity"))
		.arg(Arg::with_name("debug")
			.short("d")
			.help("print debug information"))
        .get_matches();
}

fn  load_config(c1: &mut Config) -> Result<&mut Config, config::ConfigError> {
	let c2 = c1.merge(config::Environment::with_prefix("APPLY"))?;
	let c3 = c2.merge(config::File::with_name("apply").required(false));
	c3
}
	fn main1() -> Result<(), Error> {
    dotenv::dotenv().ok();
    env_logger::init();

	let matches = arguments();

	let c1 = &mut config::Config::default();
	let conf = load_config(c1)?;
	let script_dir = match conf.get_str("script_dir") {
		Ok(val) => PathBuf::from(val),
		Err(_) => dirs::home_dir().unwrap_or(std::env::current_dir().unwrap())
	};
	println!("script dir: {:?}", script_dir);
	let script_path = PathBuf::from(script_dir);

	let action = if matches.is_present("apply") {
		Action::Apply
	} else if matches.is_present("unapply") {
		Action::UnApply
	} else if matches.is_present("show") {
		Action::Show
	} else { 
		Action::Usage
	};
	if action == Action::Usage {
		matches.usage();
		error!("usage");
	}
	let name =  matches.value_of("name").unwrap();
	//let ac = read_or_create_config("apply.toml");
	let name_config = conf.get_table(name);

	if action == Action::Apply {
		let script = format!("{}-{}.sh", name, "apply");
		let path = script_path.join(script);
		trace!("script{:?}", path);
		if !path.exists() {
			println!("create file {:?}", path);
		}
		println!("apply script {:?}", path);
		apply(name);
	}
	debug!("apply {:#?}", name );
    debug!("params {:#?}", name_config );
    Ok(())
}


fn apply(_name: &str) -> () {
}

fn main() {
	let code = match main1() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
			-1
        }
    };
	std::process::exit(code);
}

fn execute(cmd: &str) -> Result<(), ApplyError> {
    let parts: Vec<&str> = cmd.split(' ').collect();
    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .expect("cmd failed");
    println!("{} {}", Green.paint("LIVE: run "), Green.paint(cmd));
    io::stdout()
        .write_all(&output.stdout)
        .expect("error writing to stdout");
    match output.status.code() {
        Some(n) => {
            if n == 0 {
                println!(
                    "{} {}",
                    Green.paint("status code: "),
                    Green.paint(n.to_string())
                );
                Ok(())
            } else {
				println!(
                    "{} {}",
                    Red.paint("status code: "),
                    Red.paint(n.to_string())
                );
                Err(ApplyError::NotZeroExit(n))
            }
        }
        None => Err(ApplyError::CmdExitedPrematurely),
    }
}