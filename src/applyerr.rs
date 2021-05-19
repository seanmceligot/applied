extern crate thiserror;

use self::thiserror::Error;
use ansi_term::Colour;
use std::fmt;
use std::fmt::Debug;
use std::path::Path;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Warnings")]
    Warn,

    #[error("Variable not found {0}")]
    VarNotFound(String),

    #[error("Terminated without status code: ")]
    CmdExitedPrematurely,

    #[error("Non zero exit status code {0} ")]
    NotZeroExit(i32),

    #[error("Io Error {0}")]
    IoError(#[from] std::io::Error),

    #[error("Command not found {0}")]
    CommandNotFound(String),

    #[error("Expected argument: {0}")]
    ExpectedArg(&'static str),

    #[error("Insufficient Privileges {0}")]
    InsufficientPrivileges(String),

    // #[error("Path not found {0}")]
    // PathNotFound(String),
    #[error("Path not found")]
    PathNotFound0,
}
#[derive(Debug, Copy, Clone)]
pub enum Verb {
    WOULD,
    LIVE,
    SKIPPED,
}
impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self)
        Debug::fmt(self, f)
    }
}
fn color_from_verb(verb: Verb) -> Colour {
    match verb {
        Verb::WOULD => Colour::Yellow,
        Verb::LIVE => Colour::Green,
        Verb::SKIPPED => Colour::Yellow,
    }
}
pub fn log_cmd_action(action: &'static str, verb: Verb, cli: String) {
    let color: Colour = color_from_verb(verb);
    println!(
        "{}: {}: {}",
        color.paint(verb.to_string()),
        color.paint(action),
        color.paint(cli),
    );
}
pub fn log_path_action(action: &'static str, verb: Verb, path: &Path) {
    let color: Colour = color_from_verb(verb);
    println!(
        "{}: {}: {}",
        color.paint(verb.to_string()),
        color.paint(action),
        color.paint(path.display().to_string()),
    );
}
