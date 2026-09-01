//! # PARI/GP
//! 
//! This crate is meant to be used using a `gp!{}` macro and added to your project using the
//! `pari-gp` crate. Nevertheless, if you don't want to use the macro and you only want to run
//! `gp` code with &str inputs, feel free to just import this crate.

use std::{io::Write, process::{Command, Stdio}};

pub use gp_type::{FromGp, IntoGp, ParseError};
mod gp_type;

pub use gp_error::GpError;
mod gp_error;

pub fn try_from_gp<'s, T: FromGp>(input: &'s str) -> Result<T, ParseError<'s>> {
    FromGp::try_from(input).map(|(out, _)| out)
}

pub fn run(code: &str) -> Result<String, GpError> {
    let mut command = Command::new("gp");
    
    command
        .arg("-q")
        .args(["-D", "histfile=\"\""])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(spawned) => spawned,
        Err(err) => {
            let errmsg = match err.kind() {
                std::io::ErrorKind::NotFound => "Could not find `gp` on PATH. Install PARI/GP externally and make sure it's path is set in the PATH environment variable.".to_string(),
                _ => format!("Could not start PARI/GP due to an unknown error: {err}"),
            };

            return Err(GpError::Any(errmsg));
        }
    };

    child.stdin.as_mut().unwrap().write_all(code.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    if !stderr.trim().is_empty() {
        Err(GpError::from_stderr(stderr))
    } else {
        Ok(stdout)
    }
}