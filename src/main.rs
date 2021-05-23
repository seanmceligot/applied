#[macro_use]
extern crate log;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate clap;
//#[macro_use]
extern crate config;
extern crate dirs;
extern crate serde_derive;
extern crate toml;

use ansi_term::Colour::{Green, Red, Yellow};
use clap::{App, Arg};
use config::Config;
use failure::Error;
use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};
mod applyerr;
use applyerr::ApplyError;
mod action;
use action::Action;


fn arguments<'a>() -> clap::ArgMatches<'a> {
    
    let action_apply = "apply";
    let action_unapply = "unapply";
    let action_show = "show";
    let action_is_applied = "is_applied";

    //let actions = vec![action_show, action_apply, action_unapply, action_is_applied];
    let not_apply =vec![action_show, action_unapply, action_is_applied];
    let not_unapply =vec![action_show, action_apply, action_is_applied];
    let not_show =vec![ action_apply, action_unapply, action_is_applied];
    let not_is_applied =vec![action_show, action_apply, action_unapply];
    

    return App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .help(crate_description!())
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .required(true)
                .help("name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(action_unapply)
                .short("u")
                .long(action_unapply)
                .conflicts_with_all(&not_unapply)
                .help("call unapply script"),
        )
        .arg(
            Arg::with_name(action_show)
                .help("call unapply script")
                .long(action_show)
                .conflicts_with_all(&not_show)
                .help("show parameters")
        )
        .arg(
            Arg::with_name(action_apply)
                .short("a")
                .long(action_apply)
                .conflicts_with_all(&not_apply)
                .help("call apply if not applied script")
        )
        .arg(
            Arg::with_name(action_is_applied)
                .short("i")
                .long("is")
                .conflicts_with_all(&not_is_applied)
                .help("return 0 if already applied")

        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .takes_value(true)
                .help("verbosity"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("print debug information"),
        )
        .get_matches();
}

fn load_config(c1: &mut Config) -> Result<&mut Config, config::ConfigError> {
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
        Err(_) => dirs::home_dir().unwrap_or(std::env::current_dir().unwrap()),
    };
    println!("script dir: {:?}", script_dir);
    let script_path = PathBuf::from(script_dir);

    let action = if matches.is_present("apply") {
        Action::Apply
    } else if matches.is_present("is_applied") {
        Action::IsApplied
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
    let name = matches.value_of("name").unwrap();
    debug!("name {:#?}", name);
    //let ac = read_or_create_config("apply.toml");

    if action == Action::IsApplied {
        let maybe_name_config = conf.get_table(name)?;
        debug!("maybe_name_config {:#?}", maybe_name_config);
        let script = format!("{}-{}.sh", name, "is-applied");
        let path = script_path.join(script);
        trace!("script{:?}", path);
        if !path.exists() {
            println!("create file {:?}", path);
        }
        println!("apply script {:?}", path);
        let mut name_config: HashMap<String, String> = HashMap::new();
        for (k, v) in maybe_name_config {
            name_config.insert(k, v.into_str().unwrap());
        }
        debug!("params {:#?}", name_config);
        is_applied(name, &path, name_config);
    }
    if action == Action::Apply {
        let maybe_name_config = conf.get_table(name)?;
        debug!("maybe_name_config {:#?}", maybe_name_config);
        let script = format!("{}-{}.sh", name, "apply");
        let path = script_path.join(script);
        trace!("script{:?}", path);
        if !path.exists() {
            println!("create file {:?}", path);
        }
        println!("apply script {:?}", path);
        let mut name_config: HashMap<String, String> = HashMap::new();
        for (k, v) in maybe_name_config {
            name_config.insert(k, v.into_str().unwrap());
        }
        debug!("params {:#?}", name_config);
        is_applied(name, &path, name_config);
    }
    Ok(())
}

fn execute_script(cmd: &PathBuf, vars: HashMap<String, String>) -> Result<(), ApplyError> {
    let cmdstr = cmd.as_os_str();
    debug!("run: {:#?}", cmdstr);
    let output = Command::new(cmdstr)
        .envs(vars)
        .output()
        .expect("cmd failed");
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

fn is_applied(_name: &str, script: &PathBuf, vars: HashMap<String, String>) -> bool {
    match execute_script(script, vars) {
        Ok(_) => {
            println!("{}", Green.paint("Applied"));
            true
        }
        Err(_e) => {
            println!("{}", Yellow.paint("Unapplied"));
            false
        }
    }
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
