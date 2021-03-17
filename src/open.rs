use std::process::Command;
use std::{env, io};
use std::{fmt::format, path::PathBuf};
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
}

pub fn open(opts: OpenOpts) -> io::Result<()> {
    println!("Opening project: {}", opts.name);

    let mut path = super::get_dir();
    path.push(&opts.name);

    let cd_project = format!("cd {}", path.to_str().unwrap());

    let self_path = env::current_exe()?;
    let watch_cmd = format!(
        "cd {} && {} __watch {}",
        path.to_str()
            // FIXME: Deal with this better
            .expect("Path to playground cannot be converted to a string"),
        self_path
            .to_str()
            // FIXME: Deal with this better
            .expect("Path to cargo-playground cannot be converted to a string"),
        opts.name
    );

    #[rustfmt::skip]
    Command::new("tmux")
        .args(&[
            "split-window", "-h", ";",
            "send-keys", &cd_project, "&&", &watch_cmd, "C-m", ";",
            "select-pane", "-L",
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
        "select-pane", "-R", ";",
        "send-keys", "C-c", "tmux kill-pane", "C-m",
    ]).output()?;

    Ok(())
}
