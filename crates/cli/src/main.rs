use clap::{Parser, Subcommand};
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ectropy", version = version())]
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
    Cookbook {
        entry: Option<String>,
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
    if let Err(error) = run() {
        if error.broken() {
            return;
        }
        eprintln!("ectropy: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let command = match cli.command {
        Some(Command::Skill { deed }) => {
            let code = skill::run(deed);
            if code != 0 {
                std::process::exit(code);
            }
            return Ok(());
        }
        command => command,
    };
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    match command {
        Some(Command::Cookbook { entry }) => {
            let text = cookbook::render(entry.as_deref()).map_err(Error::note)?;
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
                std::process::exit(1);
            }
        }
    }
    out.flush()?;
    Ok(())
}
