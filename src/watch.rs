// A lot of this is taken from cargo-watch (https://github.com/passcod/cargo-watch/).
//
// It has been copied here so cargo-playground can be used without cargo-watch needing to be
// installed
use std::{path::MAIN_SEPARATOR, time::Duration};

use crossterm::style::Stylize;
use watchexec::{
    config::{Config, ConfigBuilder},
    error::Result,
    pathop::PathOp,
    run::{ExecHandler, Handler, OnBusyUpdate},
};

pub struct CwHandler<'a> {
    project_id: &'a str,
    inner: ExecHandler,
}

impl Handler for CwHandler<'_> {
    fn args(&self) -> Config {
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
    pub fn new(mut config: Config, project_id: &'a str) -> Result<Self> {
        let mut cmd = config.cmd.join(" && ");

        #[cfg(unix)]
        cmd.push_str("; echo [Finished running. Exit status: $?]");
        #[cfg(windows)]
        cmd.push_str(" & echo [Finished running. Exit status: %ERRORLEVEL%]");
        #[cfg(not(any(unix, windows)))]
        cmd.push_str(" ; echo [Finished running]");
        // ^ could be wrong depending on the platform, to be fixed on demand

        config.cmd = vec![cmd];

        Ok(Self {
            project_id,
            inner: ExecHandler::new(config)?,
        })
    }

    fn start(&self) {
        println!("project: {}", self.project_id.bold());
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

    let args = ConfigBuilder::default()
        .on_busy_update(OnBusyUpdate::Restart)
        .clear_screen(true)
        .run_initially(true)
        .no_environment(true)
        .poll_interval(Duration::from_millis(500))
        .debounce(Duration::from_millis(500))
        .paths(vec![".".into()])
        .ignores(ignores)
        .cmd(vec!["cargo run -q".into()])
        .build()
        .unwrap();

    let handler = CwHandler::new(args, project_id).expect("Failed to create CwHandler");
    watchexec::watch(&handler).expect("Failed to watch source files");
}
