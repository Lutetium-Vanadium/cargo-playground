use super::OpenBackend;
use crate::error;
use crate::EditorOpts;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct Gui {
    no_w: bool,
}

impl Gui {
    pub fn new(no_w: bool) -> Self {
        Self { no_w }
    }
}

impl OpenBackend for Gui {
    fn run(&mut self, mut path: PathBuf, name: &str, opts: EditorOpts) -> error::Result<()> {
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

        if !self.no_w {
            editor.arg("-w");
        }

        editor.args(opts.args).arg(&path).output()?;

        // Ignore error if user already killed it
        let _ = watch_child.kill();

        Ok(())
    }
}
