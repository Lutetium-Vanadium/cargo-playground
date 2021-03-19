use super::{path_to_str, OpenBackend};
use crate::error;
use crate::EditorOpts;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct Tmux;

impl OpenBackend for Tmux {
    fn run(mut path: PathBuf, name: &str, opts: EditorOpts) -> error::Result<()> {
        let cd_project = format!("cd {}", path_to_str(&path, "playground")?);

        let self_path = env::current_exe()?;
        let watch_cmd = format!(
            "{} && {} watch {}",
            cd_project,
            path_to_str(&self_path, "cargo-playground")?,
            name
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
        path.clear(); // Now represents path to entrypoint (main.rs)

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
}
