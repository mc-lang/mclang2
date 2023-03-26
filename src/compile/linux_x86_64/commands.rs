use std::path::{PathBuf, Path};
use std::process::{Command, Stdio};
use color_eyre::Result;
use crate::compile::Folders;
use crate::info;

pub fn linux_x86_64_compile_and_link(folders: &Folders, quiet: bool) -> Result<()> {
    
    let nasm_args = [
        "-felf64",
        folders.of_a.to_str().unwrap(),
        "-o",
        folders.of_o.to_str().unwrap()
    ];

    let ld_args = [
        folders.of_o.to_str().unwrap(),
        "-o",
        folders.of_c.to_str().unwrap()
    ];


    let mut proc = if cfg!(target_os = "windows") {
        return Ok(());
    } else {
        Command::new("nasm")
                .args(nasm_args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
    };
    if !quiet { 
        info!("running 'nasm {}'", nasm_args.join(" "));
    }
    let exit = proc.wait()?;

    if !quiet {
        info!("nasm process exited with code {}", exit);
    }


    let mut proc2 = if cfg!(target_os = "windows") {
        return Ok(());
    } else {
        Command::new("ld")
                .args(ld_args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
    };
    if !quiet {
        info!("running 'ld {}'", ld_args.join(" "));
    }
    let exit2 = proc2.wait()?;
    if !quiet {
        info!("ld process exited with code {}", exit2);
    }
    

    
    Ok(())
}

pub fn linux_x86_64_run(bin: &Path, args: &[String], quiet: bool) -> Result<i32> {

    let bin = PathBuf::from(
        format!("./{}", bin.to_string_lossy())
    );

    let mut proc = if cfg!(target_os = "windows") {
        return Ok(0);
    } else {
        Command::new(bin.clone())
                .args(args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
    };
    // println!("{}", quiet);
    if !quiet {
        info!("running {} {}", bin.to_string_lossy(), args.join(" "));
    }
    let exit = proc.wait()?;
    if !quiet {
        info!("{} process exited with code {}", bin.to_string_lossy(), exit);
    }

    Ok(exit.code().unwrap_or(0))
}