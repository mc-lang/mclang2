
use std::path::{PathBuf, Path};
use std::process::Stdio;
use std::{process, fs};
use clap::Parser;
use anyhow::{Result, bail};

pub mod color {
    #![allow(dead_code)]
    pub const NONE: &str = "\x1b[0m";
    pub const RESET: &str = "\x1b[0m";
    pub const BRIGHT: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const UNDERSCORE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const HIDDEN: &str = "\x1b[8m";
    pub const FG_BLACK: &str = "\x1b[30m";
    pub const FG_RED: &str = "\x1b[31m";
    pub const FG_GREEN: &str = "\x1b[32m";
    pub const FG_YELLOW: &str = "\x1b[33m";
    pub const FG_BLUE: &str = "\x1b[34m";
    pub const FG_MAGENTA: &str = "\x1b[35m";
    pub const FG_CYAN: &str = "\x1b[36m";
    pub const FG_WHITE: &str = "\x1b[37m";
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_MAGENTA: &str = "\x1b[45m";
    pub const BG_CYAN: &str = "\x1b[46m";
    pub const BG_WHITE: &str = "\x1b[47m";
}

#[allow(dead_code)]
#[derive(Debug)]
struct TestOutput {
    stdout: String,
    stderr: String,
    stdin: String,
    status: i32
}

fn run_test<P: Into<PathBuf> + std::convert::AsRef<std::ffi::OsStr>>(f_in: PathBuf, f_out: &PathBuf, compiler: P, compile_mode: bool, stdin: String) -> Result<TestOutput> {
    let mut command = process::Command::new(compiler);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if compile_mode {
        command.arg("-cqr");
    } else {
        command.arg("-sq");
    }

    command.arg("-i");
    command.arg(f_in);
    command.arg("-o");
    command.arg(f_out);

    let child = command.spawn()?;

    let out = child.wait_with_output()?;
    
    let stdout = out.stdout.iter().map(|c| {
        char::from_u32(u32::from(*c)).expect("Failed to parse stdout char").to_string()
    }).collect::<String>();

    let stderr = out.stderr.iter().map(|c| {
        char::from_u32(u32::from(*c)).expect("Failed to parse stderr char").to_string()
    }).collect::<String>();


    Ok(TestOutput {
        stdout,
        stderr,
        stdin,
        status: out.status.code().unwrap()
    })
}

fn run_tests(args: Args) -> Result<()>{

    let files = fs::read_dir(args.input)?;

    for file in  files {
        let file = file?;
        let f_name = file.file_name().to_string_lossy().to_string();
        let f_out = PathBuf::from(&args.output).join(f_name);


        let intp = run_test(file.path(), &f_out, &args.compiler_path, false, String::new())?;
        let comp = run_test(file.path(), &f_out, &args.compiler_path, true, String::new())?;
        compare_results(&intp, &comp, &file.path())?;
    }

    Ok(())
}


fn compare_results(intp: &TestOutput, comp: &TestOutput, f_in: &Path) -> Result<()> {

    if intp.stdout != comp.stdout {
        println!("{b}[ {r}ERR{rs}{b} ]{rs} {f} compiled and interpreted stdout versions differ", r=color::FG_RED, rs=color::RESET, b=color::BRIGHT, f=f_in.display());
        println!("compiled:\n{}", comp.stdout);
        println!("interpreted:\n{}", intp.stdout);
        bail!("Testing failed");
    }

    if intp.stderr != comp.stderr {
        println!("{b}[ {r}ERR{rs}{b} ]{rs} {f} compiled and interpreted stderr versions differ", r=color::FG_RED, rs=color::RESET, b=color::BRIGHT, f=f_in.display());
        println!("compiled:\n{}", comp.stderr);
        println!("interpreted:\n{}", intp.stderr);
        bail!("Testing failed");
    }

    if intp.status != comp.status {
        println!("{b}[ {r}ERR{rs}{b} ]{rs} {f} compiled and interpreted status codes differ", r=color::FG_RED, rs=color::RESET, b=color::BRIGHT, f=f_in.display());
        println!("compiled:\n{}", comp.status);
        println!("interpreted:\n{}", intp.status);
        bail!("Testing failed");
    }

    println!("{b}[ {g}OK{rs}{b} ]{rs} {f} ", g=color::FG_GREEN, rs=color::RESET, b=color::BRIGHT, f=f_in.display());
    Ok(())
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Mode, allowed modes: test, record
    #[arg(long, short)]
    mode: String,
    
    /// Use compile mode
    #[arg(long, short)]
    compile: bool,
    
    /// Use interpret mode
    #[arg(long, short='s')]
    interpret: bool,
    
    /// Output folder
    #[arg(long, short, default_value_t=String::from("./target/mcl_test_dev"))]
    output: String,
    
    /// Input folder
    #[arg(long, short, default_value_t=String::from("./tests"))]
    input: String,

    /// Compiler path
    #[arg(long, short, default_value_t=String::from("./target/release/mclangc"))]
    compiler_path: String


}

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.output)?;
    match args.mode.as_str() {
        "test" => run_tests(args),
        "record" => todo!("Implement test result recording"),
        s => {
            eprintln!("Unknown mode '{s}'");
            bail!("Bad subcommand");
        }
    }?;

    Ok(())
}
