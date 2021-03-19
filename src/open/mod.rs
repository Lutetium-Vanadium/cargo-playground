use crate::error;
use std::path::{Path, PathBuf};
use std::{env, io};
use structopt::StructOpt;

mod gui;
mod tmux;

trait OpenBackend {
    fn run(path: PathBuf, name: &str, opts: crate::EditorOpts) -> error::Result<()>;
}

#[derive(StructOpt, Debug)]
pub struct OpenOpts {
    #[structopt(flatten)]
    pub(crate) editor_opts: super::EditorOpts,
    /// The name of the playground to open
    pub(crate) name: String,
    /// Indicates the editor is a gui editor
    #[structopt(short, long)]
    pub gui: bool,
    #[structopt(skip = false)]
    pub(crate) skip_check: bool,
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

    if opts.gui {
        println!("opening project: {}", opts.name);
        gui::Gui::run(path, &opts.name, opts.editor_opts)
    } else if env::var_os("TMUX").is_some() {
        println!("opening project: {}", opts.name);
        tmux::Tmux::run(path, &opts.name, opts.editor_opts)
    } else {
        Err(error::Error::new(
            io::ErrorKind::Other,
            "currently only terminals running tmux are supported",
        )
        .with_help("try using the --force flag with a GUI editor"))
    }
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
