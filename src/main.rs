use std::path::PathBuf;
use std::{env, io};
use structopt::StructOpt;

mod new;
mod open;
mod watch;

#[derive(StructOpt, Debug)]
/// Make and use playgrounds locally.
enum Opts {
    /// Creates a new playground
    New(new::NewOpts),
    /// Creates a new playground
    Open(open::OpenOpts),
    /// List currently existing playgrounds
    Ls,
}

fn get_dir() -> PathBuf {
    env::temp_dir().join("cargo-playground")
}

fn main() {
    // FIXME: print errors in a better format
    match run() {
        Ok(()) => {}
        Err(e) => eprintln!("{:?}", e),
    }
}

fn run() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    // FIXME: See if this can be moved into Opts without a help message being displayed for it
    // This can technically be accessed from the binary, but it would not be a good idea to use it
    // since it may easily crash if the setup isn't exactly how it expects.
    if let Some("__watch") = args.get(1).map(String::as_str) {
        watch::watch(&args[2]);

        return Ok(());
    }

    let opts = Opts::from_iter(args);

    if env::var_os("TMUX").is_none() {
        eprintln!("Currently only terminals running tmux are supported");
    }

    match opts {
        Opts::New(opts) => {
            new::create(opts)?;
        }
        Opts::Open(opts) => {
            open::open(opts)?;
        }
        Opts::Ls => {
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
