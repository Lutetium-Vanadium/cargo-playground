use crate::error;
use std::io;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod tmux;

trait OpenBackend {
    fn run(path: PathBuf, opts: OpenOpts) -> error::Result<()>;
}

#[derive(StructOpt, Debug)]
pub struct OpenOpts {
    #[structopt(short, long, env = "VISUAL", hide_env_values = true)]
    /// The editor to open the project in
    pub editor: String,
    #[structopt(short, long)]
    /// Extra args (if any) to be supplied to the editor
    pub args: Vec<String>,
    /// The name of the playground to open
    pub name: String,
    #[structopt(skip = false)]
    pub skip_check: bool,
}

pub fn open(opts: OpenOpts) -> error::Result<()> {
    let mut path = super::get_dir();
    path.push(&opts.name); // Now represents playground path

    if !opts.skip_check && !path.is_dir() {
        return Err(error::Error::new(
            io::ErrorKind::NotFound,
            format!("could not find playground with name {:?}", path),
        )
        .with_help(
            "use `cargo playground ls` to list available playgrounds
       or `cargo playground new` to create a new playground",
        ));
    }

    println!("opening project: {}", opts.name);

    tmux::Tmux::run(path, opts)
}

fn path_to_str<'a>(path: &'a Path, path_name: &str) -> io::Result<&'a str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "could not convert {} path {:?} to a utf-8 string",
                path_name, path
            ),
        )
    })
}
