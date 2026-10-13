use clap::{Parser, Subcommand};
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ectropy",
    version = version(),
    after_help = "Optional Locus observation: set [locus] enabled = true in config.toml below the existing Ectropy data home. endpoint defaults to LOCUS_API or http://127.0.0.1:43308; buffer uses state/locus below that home. Missing configuration disables observation. Repository ectropy.toml remains scan policy."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(default_value = ".")]
    root: PathBuf,
}

#[derive(Subcommand)]
enum Command {
    Shape {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Vocabulary {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Law {
        name: Option<String>,
    },
    Cookbook {
        code: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Skill {
        #[command(subcommand)]
        deed: skill::Deed,
    },
}

fn version() -> &'static str {
    plumb::version!("ECTROPY")
}

mod cookbook;
mod law;
mod observation;
mod repo;
mod report;
mod rig;
mod skill;
use repo::Repo;

#[derive(Debug)]
pub(crate) enum Error {
    Io(io::Error),
    Note(String),
}

impl Error {
    pub(crate) fn note(note: impl Into<String>) -> Self {
        Self::Note(note.into())
    }

    fn broken(&self) -> bool {
        matches!(self, Self::Io(source) if source.kind() == io::ErrorKind::BrokenPipe)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => source.fmt(out),
            Self::Note(note) => out.write_str(note),
        }
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

fn main() {
    if let Err(error) = plumb::identity!("ECTROPY") {
        eprintln!("ectropy: {error}");
        std::process::exit(2);
    }
    let cli = Cli::parse();
    let observation = observation::Run::start(observation::name(&cli.command));
    let code = match execute(cli) {
        Ok(code) => code,
        Err(error) if error.broken() => 0,
        Err(error) => {
            eprintln!("ectropy: {error}");
            2
        }
    };
    if let Some(observation) = observation {
        observation.finish(code);
    }
    if code != 0 {
        std::process::exit(code);
    }
}

#[locus::trace(with = observation::view())]
fn execute(cli: Cli) -> Result<i32, Error> {
    plumb::identity::ready().map_err(Error::note)?;
    let command = match cli.command {
        Some(Command::Skill { deed }) => {
            return Ok(skill::run(deed));
        }
        command => command,
    };
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    match command {
        Some(Command::Cookbook { code, json }) => {
            let text = cookbook::render(code.as_deref(), json).map_err(Error::note)?;
            out.write_all(text.as_bytes())?;
        }
        Some(Command::Law { name }) => {
            let text = law::render(name.as_deref()).map_err(Error::note)?;
            out.write_all(text.as_bytes())?;
        }
        Some(Command::Shape { root }) => {
            let repo = Repo::open(&root)?;
            repo.shape(&repo.files()?, &mut out)?;
        }
        Some(Command::Vocabulary { root }) => {
            let repo = Repo::open(&root)?;
            repo.vocabulary(&repo.files()?, &mut out)?;
        }
        Some(Command::Skill { .. }) => unreachable!(),
        None => {
            let repo = Repo::open(&cli.root)?;
            let findings = repo.scan(&repo.files()?)?;
            if report::write(&findings, &mut out)? {
                out.flush()?;
                return Ok(1);
            }
        }
    }
    out.flush()?;
    Ok(0)
}
