#[macro_use]
extern crate log;
extern crate dotenv;
extern crate env_logger;
extern crate seahorse;
//#[macro_use]
extern crate config;
extern crate dirs;
extern crate serde_derive;
extern crate toml;
extern crate tempfile;

use ansi_term::Colour::{Green, Red, Yellow};
use config::Config;
use failure::Error;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
mod applyerr;
use applyerr::ApplyError;
///mod action;

enum Script {
    FsPath(PathBuf),
    InMemory(String)
}

fn load_config(c1: &mut Config) -> Result<&mut Config, config::ConfigError> {
    let c2 = c1.merge(config::Environment::with_prefix("APPLY"))?;
    let c3 = c2.merge(config::File::with_name("apply").required(false));
    c3
}
fn main1() -> Result<(), Error> {
    dotenv::dotenv().ok();
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let apply_command = seahorse::Command::new("apply")
    .description("apply [name] if not already applied")
    .alias("a")
    .usage("apply(a) [name...]")
    .action(apply_action);

    let is_applied_command = seahorse::Command::new("is_applied")
    .description("is_applied [name] if not already applied")
    .alias("i")
    .usage("is_applied(i) [name...]")
    .action(is_applied_action);
  
    let app = seahorse::App::new(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .version(env!("CARGO_PKG_VERSION"))
    .usage("applied action name")
    .action(apply_action)
    .command(apply_command)
    .command(is_applied_command);

    app.run(args);
    
  
    Ok(())
}
fn get_script_directory(conf: &mut Config) -> PathBuf {
    let script_dir = match conf.get_str("script_dir") {
        Ok(val) => PathBuf::from(val),
        Err(_) => dirs::home_dir().unwrap_or(std::env::current_dir().unwrap()),
    };
    println!("script dir: {:?}", script_dir);
    let script_path = PathBuf::from(script_dir);
    if !script_path.exists() {
        panic!("{:?} does not exist", script_path);
    }
    script_path
}
#[test]
fn test_appply() -> Result<(), Error> {
    let apply_script = Script::InMemory(String::from("touch test1.tmp"));
    let is_applied = Script::InMemory(String::from("test -f test1.tmp"));
    let name = "example1";   

    let name_config : HashMap<String,String>  = HashMap::new(); 
    do_is_applied(name_config.clone(), &is_applied, name)?; 
    do_apply(name_config,&apply_script, name)?;    
    Ok(())    
}
fn apply_action(c: &seahorse::Context) {
    println!("apply_action");
    let name: &str = c.args.first().unwrap();
    debug!("apply_action {}", name);

    let c1 = &mut config::Config::default();
    let conf = load_config(c1).unwrap();

    let name_config: HashMap<String, String> = scriptlet_config(conf, name).expect("scriptlet_config");
    let is_applied_script = find_scriptlet(conf, name, "is-applied");
    do_is_applied(name_config.clone(), &is_applied_script, name).unwrap();    
    let apply_script = find_scriptlet(conf, name, "apply");
    do_apply(name_config, &apply_script, name).unwrap();    

}
fn is_applied_action(c: &seahorse::Context) {
    println!("is_applied_action");
    let name: &str = c.args.first().unwrap();
    debug!("is_applied_action {}", name);
    
    let c1 = &mut config::Config::default();
    let conf = load_config(c1).unwrap();
    let name_config: HashMap<String, String> = scriptlet_config(conf, name).expect("scriptlet_config");

    let is_applied_script = find_scriptlet(conf, name, "is-applied");

    do_is_applied(name_config, &is_applied_script, name).unwrap();
}
fn find_scriptlet(conf: &mut Config, name: &str, action: &str) -> Script {
    let filename = format!("{}-{}", name, action);
    debug!("script filename {}", filename);
    let dir = get_script_directory(conf);
    debug!("dir {:?}", dir);
    let path = dir.join(filename);
    trace!("script{:?}", path);
    if !path.exists() {
        println!("create file {:?}", path);
    }
    println!("apply script {:?}", path);
    Script::FsPath(path)
}
fn do_apply(name_config: HashMap<String,String>, script_path: &Script, name: &str) -> Result<(), Error> {
    
    debug!("params {:#?}", name_config);
    execute_apply(name, script_path, name_config);
    Ok(())
}

fn do_is_applied(name_config: HashMap<String,String>, script_path: &Script, name: &str) -> Result<(), Error> {
    debug!("params {:#?}", name_config);
    is_applied(name, &script_path, name_config);
    Ok(())
}
fn scriptlet_config(conf: &mut Config, name: &str) -> Result<HashMap<String, String>, Error> {
    let maybe_name_config: HashMap<String, config::Value> = conf.get_table(name)?;
    debug!("maybe_name_config {:#?}", maybe_name_config);
    let mut name_config: HashMap<String, String> = HashMap::new();
    for (k, v) in maybe_name_config {
        name_config.insert(k, v.into_str().unwrap());
    }
    Ok(name_config)
}


fn execute_script_file(cmdpath: &Path,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
    let cmdstr = cmdpath.as_os_str();
    debug!("run: {:#?}", cmdstr);
    let output = Command::new("bash")
        .arg(cmdstr)
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
fn execute_script(script: &Script,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
    match script {
        Script::FsPath(path) => execute_script_file(path,vars),
        Script::InMemory(source) => {
            let mut t = tempfile::NamedTempFile::new().unwrap();
            t.write(source.as_bytes()).unwrap();
            debug!("execute {:?}", t.path());
            let r = execute_script_file(t.path(), vars);
            t.close().unwrap();
            r
        }
    }
}
fn execute_apply(_name: &str, script: &Script, vars: HashMap<String, String>) -> bool {
    match execute_script(script, vars) {
        Ok(_) => {
            println!("{}", Green.paint("Applied"));
            true
        }
        Err(_e) => {
            println!("{}", Yellow.paint("Apply Failed"));
            false
        }
    }
}
fn is_applied(_name: &str, script: &Script, vars: HashMap<String, String>) -> bool {
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
