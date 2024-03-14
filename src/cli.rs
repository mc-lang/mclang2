use clap::{builder::PossibleValue, Parser, ValueEnum};
use camino::Utf8PathBuf;
lazy_static::lazy_static! {
    static ref DEFAULT_INCLUDE_PATHS: Vec<Utf8PathBuf> = vec![
        Utf8PathBuf::from("./"),
        Utf8PathBuf::from("./include"),
        Utf8PathBuf::from("~/.mclang/include"),
    ];
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Only compile, dont link
    #[arg(long, short)]
    pub compile: bool, 

    /// Verosity
    /// -1 - Nothing
    /// 0  - Only errors
    /// 1  - Normal
    /// 2  - Verbose
    /// 3  - Tracing
    #[arg(long, short, default_value_t=1)]
    pub verbose: i8,

    /// Runt the program after compilation
    #[arg(long, short)]
    pub run: bool,

    /// Output execuable file path
    #[arg(long, short, default_value="./a.out")]
    pub output: Utf8PathBuf,
    
    /// Paths to search for libraries
    #[arg(long="include", short='I', default_values_t=DEFAULT_INCLUDE_PATHS.clone().into_iter())]
    pub include_path: Vec<Utf8PathBuf>, 

    /// Target to compile to
    #[arg(long, short='T', default_value_t=CompilationTarget::X86_64_linux_nasm)]
    pub target: CompilationTarget,

    /// Input code files
    pub input: Vec<Utf8PathBuf>,
    
    #[clap(skip)]
    pub passthrough: Vec<String>
}

impl CliArgs {
    pub fn parse_with_passthrough() -> Self {
        let mut clap_args = Vec::new();
        let mut pt_args = Vec::new();
        let mut switch = false;
        for arg in std::env::args() {
            if arg == String::from("--") {
                switch = true;
                continue;
            }

            if !switch {
                //clap args
                clap_args.push(arg);
            } else {
                // passwthrough
                pt_args.push(arg);
            }
        }

        let mut cargs = Self::parse_from(clap_args);
        cargs.passthrough = pt_args;

        cargs
    }
}



#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum CompilationTarget {
    X86_64_linux_nasm
}

impl ValueEnum for CompilationTarget {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::X86_64_linux_nasm
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            CompilationTarget::X86_64_linux_nasm => Some(PossibleValue::new("x86_64-linux-nasm")),
        }
    }
}

impl std::fmt::Display for CompilationTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let r = match self {
            CompilationTarget::X86_64_linux_nasm => "x86_64-linux-nasm",
        };
        write!(f, "{}", r)
    }
}

// impl From<CompilationTarget> for clap::builder::OsStr {
//     fn from(value: CompilationTarget) -> Self {
//         match value {
//             CompilationTarget::X86_64_linux_nasm => "X86_64_linux_nasm".into()
//         }
//     }
// }

// impl TryFrom<&str> for CompilationTarget {
//     type Error = anyhow::Error;
//     fn try_from(value: &str) -> Result<Self, Error> {
//         match value {
//             "X86_64_linux_nasm" => Ok(CompilationTarget::X86_64_linux_nasm)
//             _ => bail!("Unknown compilation target {value}")
//         }
//     }
    
// }

