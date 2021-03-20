use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

mod error;
mod new;
mod open;
mod watch;

#[derive(StructOpt, Debug)]
/// Make and use playgrounds locally.
#[structopt(bin_name = "cargo", usage = "cargo playground <SUBCOMMAND>")]
enum Opts {
    // FIXME: See if this can be hidden from help message
    /// Internal command required for running the playground -- good idea not to use it
    Watch {
        playground_id: String,
    },
    Playground(PlaygroundOpts),
}

#[derive(StructOpt, Debug)]
/// Make and use playgrounds locally.
enum PlaygroundOpts {
    /// Creates a new playground
    // Override the default because it include '--editor <editor>'
    #[structopt(usage = "cargo playground new [FLAGS] [OPTIONS] [--] [dependencies]...")]
    New(new::NewOpts),
    /// Opens an already existing playground
    // Override the default because it include '--editor <editor>'
    #[structopt(usage = "cargo playground open [FLAGS] [OPTIONS] <name>")]
    Open(open::OpenOpts),
    /// List currently existing playgrounds
    #[structopt(alias = "list")]
    Ls,
}

#[derive(StructOpt, Debug)]
struct EditorOpts {
    /// The editor to open the project in
    #[structopt(short, long, env = "VISUAL", hide_env_values = true)]
    pub editor: String,
    /// Extra args (if any) to be supplied to the editor
    #[structopt(short, long)]
    pub args: Vec<String>,
}

fn get_dir() -> PathBuf {
    env::var_os("CARGO_PLAYGROUND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cargo-playground"))
}

fn main() {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support();

    match run() {
        Ok(()) => {}
        Err(e) => eprintln!("{}", e),
    }
}

fn run() -> error::Result<()> {
    let opts = Opts::from_args();

    let opts = match opts {
        Opts::Playground(opts) => opts,
        Opts::Watch { playground_id } => {
            watch::watch(&playground_id);

            return Ok(());
        }
    };

    match opts {
        PlaygroundOpts::New(opts) => {
            new::new(opts)?;
        }
        PlaygroundOpts::Open(opts) => {
            open::open(opts)?;
        }
        PlaygroundOpts::Ls => {
            let path = get_dir();
            if !path.exists() {
                return Ok(());
            }

            for entry in get_dir().read_dir()? {
                // ignoring errors for now, maybe do something about it?
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        println!("{}", name);
                    }
                }
            }
        }
    }

    Ok(())
}
