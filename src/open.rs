use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};
use structopt::StructOpt;

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

pub fn open(opts: OpenOpts) -> io::Result<()> {
    let mut path = super::get_dir();
    path.push(&opts.name);

    if !opts.skip_check && !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "could not find playground with name {:?}
help: use `cargo playground ls` to list available playgrounds",
                path,
            ),
        ));
    }

    println!("opening project: {}", opts.name);

    let cd_project = format!("cd {}", path_to_str(&path, "playground")?);

    let self_path = env::current_exe()?;
    let watch_cmd = format!(
        "{} && {} watch {}",
        cd_project,
        path_to_str(&self_path, "cargo-playground")?,
        opts.name
    );

    #[rustfmt::skip]
    Command::new("tmux")
        .args(&[
            "split-window", "-h", ";",                              // Create a right pane,
            "send-keys", &cd_project, "&&", &watch_cmd, "C-m", ";", // watch the project files
            "select-pane", "-L",                                    // and focus the editor
        ])
        .output()?;

    let mut editor = Command::new(opts.editor);

    editor.current_dir(&path);
    let mut path = PathBuf::new();

    path.push("src");
    path.push("main.rs");

    editor.args(opts.args).arg(&path).status()?;

    #[rustfmt::skip]
    Command::new("tmux").args(&[
        "select-pane", "-R", ";",                    // Select the right pane
        "send-keys", "C-c", "tmux kill-pane", "C-m", // and kill it
    ]).output()?;

    Ok(())
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
