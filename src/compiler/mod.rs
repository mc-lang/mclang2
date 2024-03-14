mod x86_64_linux_nasm;
mod utils;

use anyhow::bail;

use crate::{cli::{CliArgs, CompilationTarget}, types::ast::Program};
use std::{collections::HashMap, fs::File, io::{BufWriter, Write}, path::{Path, PathBuf}, process::Command};

use self::utils::run_cmd;


pub trait Compiler {
    fn new() -> Self;
    fn generate_asm(&mut self, prog: &Program, fd: &mut BufWriter<File>) -> anyhow::Result<()>;
    fn compile(&mut self, asm_fp: &Path, obj: &Path) -> anyhow::Result<()>;
    fn link(&mut self, obj_files: Vec<PathBuf>, bin_fp: &Path) -> anyhow::Result<()>;
    /// Return programs that are needed
    fn needed_dependencies(&mut self) -> Vec<&str>;
}

//NOTE: No bsd cause im not about to create 3 or 4 diffrent compilation targets

pub fn compile_program(cli_args: &CliArgs, prog_map: HashMap<&Path, Program>) -> anyhow::Result<()> {
    let mut compiler = match cli_args.target {
        CompilationTarget::X86_64_linux_nasm => x86_64_linux_nasm::X86_64LinuxNasmCompiler::new(),
    };
    let bin_p = cli_args.output.as_std_path();
    let mut objs = Vec::new();
    for (k, v) in prog_map {
        let mut asm_p = k.to_path_buf();
        let mut obj_p = k.to_path_buf();
        
        asm_p.set_extension("s");
        obj_p.set_extension("o");
        
        
        if let Err(_) = compile_file(&mut compiler, cli_args, asm_p.as_path(), obj_p.as_path(), &v) {
            error!("Failed to compile file {k:?}");
            bail!("")
        }
        objs.push(obj_p.clone());
    }

    if let Err(e) = compiler.link(objs, bin_p) {
        error!("Failed to link program: {e}");
        bail!("")
    }
    
    info!("Finished building program");

    if cli_args.run {
        // run_cmd(format!("./{}", bin_p.to_string_lossy()), cli_args.passthrough.clone())?;
        let bin = bin_p.to_string_lossy().to_string();
        let mut cmd = Command::new(format!("./{}", bin.clone()));
        let cmd = cmd.args(cli_args.passthrough.clone());
        
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
        } else {
            info!("Process exited successfully")
        }

    }


    Ok(())
}

pub fn compile_file<C: Compiler>(compiler: &mut C, _: &CliArgs, asm_file: &Path, obj_file: &Path, prog: &Program) -> anyhow::Result<()> {

    let asm_fd = std::fs::File::options()
        .write(true)
        .write(true)
        .create(true)
        .truncate(true)
        .append(false)
        .open(asm_file);

    let asm_fd = match asm_fd {
        Ok(fd) => fd,
        Err(e) => {
            error!("Failed to open file {asm_file:?}: {e}");
            bail!("");
        }
    };

    let mut buf_asm_fd = BufWriter::new(asm_fd);

    compiler.generate_asm(prog, &mut buf_asm_fd)?;
    buf_asm_fd.flush()?;

    compiler.compile(asm_file, obj_file)?;
    Ok(())
}