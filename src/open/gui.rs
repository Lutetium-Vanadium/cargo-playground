use super::OpenBackend;
use crate::error;
use crate::EditorOpts;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct Gui;

impl OpenBackend for Gui {
    fn run(mut path: PathBuf, name: &str, opts: EditorOpts) -> error::Result<()> {
        let self_path = env::current_exe()?;
        let mut watch_child = Command::new(self_path)
            .current_dir(&path)
            .arg("watch")
            .arg(name)
            .spawn()?;

        let mut editor = Command::new(opts.editor);

        editor.current_dir(&path);
        path.clear(); // Now represents path to entrypoint (main.rs)

        path.push("src");
        path.push("main.rs");

        editor.args(opts.args).arg(&path).status()?;

        // Ignore error if user already killed it
        let _ = watch_child.kill();

        Ok(())
    }
}
