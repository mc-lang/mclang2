use std::{fmt::Debug, process::{Command, Stdio}};

use anyhow::bail;



pub fn run_cmd<'a, S: Into<String> + Debug + Clone>(bin: S, args: Vec<String>) -> anyhow::Result<()> {
    let debug = unsafe {
        crate::logger::LOGGER.enabled(crate::logger::Level::Debug)
    };
    let mut cmd = Command::new(bin.clone().into());
    let cmd = cmd.args(args);
    let cmd = if debug {
        cmd.stdout(Stdio::inherit())
    } else {
        cmd.stdout(Stdio::null())
    };
    let cmd = cmd.stderr(Stdio::inherit());
    
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!("Unable to run {cmd:?}: {e}");
            bail!("");
        }
    };
    let ret = child.wait_with_output().expect("fuck i know");


    if !ret.status.success() {
        error!("Process running {bin:?} exited abnormaly, run with -v 2 for more output");
        bail!("")
    }

    Ok(())
}