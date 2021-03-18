use std::path::MAIN_SEPARATOR;
use watchexec::{
    cli::{Args, ArgsBuilder},
    error::Result,
    pathop::PathOp,
    run::{ExecHandler, Handler},
};

pub struct CwHandler<'a> {
    project_id: &'a str,
    inner: ExecHandler,
}

impl Handler for CwHandler<'_> {
    fn args(&self) -> Args {
        self.inner.args()
    }

    fn on_manual(&self) -> Result<bool> {
        let res = self.inner.on_manual()?;
        self.start();
        Ok(res)
    }

    fn on_update(&self, ops: &[PathOp]) -> Result<bool> {
        let res = self.inner.on_update(ops)?;
        self.start();
        Ok(res)
    }
}

impl<'a> CwHandler<'a> {
    pub fn new(mut args: Args, project_id: &'a str) -> Result<Self> {
        let mut cmd = args.cmd.join(" && ");

        #[cfg(unix)]
        cmd.push_str("; echo [Finished running. Exit status: $?]");
        #[cfg(windows)]
        cmd.push_str(" & echo [Finished running. Exit status: %ERRORLEVEL%]");
        #[cfg(not(any(unix, windows)))]
        cmd.push_str(" ; echo [Finished running]");
        // ^ could be wrong depending on the platform, to be fixed on demand

        args.cmd = vec![cmd];

        Ok(Self {
            project_id,
            inner: ExecHandler::new(args)?,
        })
    }

    fn start(&self) {
        println!(
            "project: {}",
            ansi_term::Style::new().bold().paint(self.project_id)
        );
    }
}

pub fn watch(project_id: &str) {
    let ignores = vec![
        // Mac
        format!("*{}.DS_Store", MAIN_SEPARATOR),
        // Vim
        "*.sw?".into(),
        "*.sw?x".into(),
        // Emacs
        "#*#".into(),
        ".#*".into(),
        // Kate
        ".*.kate-swp".into(),
        // VCS
        format!("*{s}.hg{s}**", s = MAIN_SEPARATOR),
        format!("*{s}.git{s}**", s = MAIN_SEPARATOR),
        format!("*{s}.svn{s}**", s = MAIN_SEPARATOR),
        // SQLite
        "*.db".into(),
        "*.db-*".into(),
        format!("*{s}*.db-journal{s}**", s = MAIN_SEPARATOR),
        // Rust
        format!("*{s}target{s}**", s = MAIN_SEPARATOR),
    ];

    let args = ArgsBuilder::default()
        .restart(true)
        .clear_screen(true)
        .run_initially(true)
        .no_environment(true)
        .poll_interval(500u32)
        .debounce(500u32)
        .paths(vec![".".into()])
        .ignores(ignores)
        .cmd(vec!["cargo run -q".into()])
        .build()
        .unwrap();

    let handler = CwHandler::new(args, project_id).expect("Failed to create CwHandler");
    watchexec::watch(&handler).expect("Failed to watch source files");
}
