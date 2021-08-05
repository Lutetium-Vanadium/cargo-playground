use structopt::StructOpt;

mod clean;
mod error;
mod helpers;
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
    /// Cleans the playgrounds directory, deleting all cargo projects in it.
    Clean(clean::CleanOpts),
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

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
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
        PlaygroundOpts::New(opts) => new::new(opts),
        PlaygroundOpts::Open(opts) => open::open(opts),
        PlaygroundOpts::Clean(opts) => clean::clean(opts),
        PlaygroundOpts::Ls => {
            let path = helpers::get_dir();
            if !path.exists() {
                return Ok(());
            }

            // ignoring errors for now, maybe do something about it?
            for entry in path.read_dir()?.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    println!("{}", name);
                }
            }

            Ok(())
        }
    }
}
