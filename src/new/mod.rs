use crate::{error, helpers, open};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{atomic, Arc};
use std::time::SystemTime;
use std::{fmt, fs, io};
use structopt::StructOpt;

mod examples;
mod pick_from;

use examples::Examples;

#[derive(StructOpt, Debug)]
pub struct NewOpts {
    /// The name of the playground to create. If not supplied, the current timestamp will be used
    #[structopt(short, long)]
    name: Option<String>,
    #[structopt(flatten)]
    editor_opts: super::EditorOpts,
    /// Do not pass -w flag when opening GUI editor
    #[structopt(long, requires("gui"))]
    no_w: bool,
    /// Indicates the editor is a gui editor
    #[structopt(short, long)]
    gui: bool,
    /// The library to base main.rs on. If not provided, base Cargo main.rs will be used.
    ///
    /// Follows same format as dependencies. You do not need to repeat the library in dependencies,
    /// as it is automatically added
    #[structopt(short, long)]
    template: Option<String>,
    /// The dependencies to add. It must be in the following format:
    /// 1. <dep-name>
    /// 2. <dep-name>=<dep-version>
    #[structopt(name = "dependencies")]
    deps: Vec<String>,
}

pub fn new(opts: NewOpts) -> error::Result<()> {
    let name = match opts.name {
        Some(name) => name,
        None => {
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|_| {
                    error::Error::new(io::ErrorKind::Other, "current time is unix epoch!")
                })?;

            format!("playground-{}", time.as_secs())
        }
    };

    helpers::print_status("Creating", &name);

    let mut path = helpers::get_dir();
    path.push(&name); // Now represents the playground directory

    if !Command::new("cargo")
        .arg("new")
        .arg(&path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success()
    {
        return Err(error::Error::new(
            io::ErrorKind::Other,
            "could not create cargo project",
        ));
    }

    path.push("Cargo.toml"); // Now represents path to Cargo.toml
    let mut cargo_toml = fs::OpenOptions::new().append(true).open(&path)?;

    if let Some(ref template) = opts.template {
        let stop = Arc::new(false.into());
        let loader = helpers::loader("fetching examples", Arc::clone(&stop));

        let dep = Dep::try_parse(&template)?;
        writeln!(cargo_toml, "{}", dep)?;

        cargo_toml.flush()?;

        let examples = Examples::find(&mut path, dep.dep_name)
            .map_err(|e| error::Error::new(e.kind(), format!("couldn't get templates: {}", e)))?
            .ok_or_else(|| {
                error::Error::new(
                    io::ErrorKind::NotFound,
                    format!("couldn't find any templates for {}", dep.dep_name),
                )
                .with_help(
                    "templates are taken from the `examples` directory in a crate
       check if the crate has an examples directory",
                )
            });

        // only one writer, and reader doesn't care about the race, so its fine
        stop.store(true, atomic::Ordering::Relaxed);
        let _ = loader.join();

        // do not propagate error until loading thread has joined
        let example = pick_from::pick_from(examples?).map_err(|err| {
            error::Error::new(
                io::ErrorKind::Other,
                format!("couldn't pick template: {}", err),
            )
        })?;

        if example.is_none() {
            return Ok(());
        }

        path.pop();
        path.push("src");
        path.push("main.rs");

        fs::copy(&example.unwrap(), &path)?;
    }

    for dep in opts.deps {
        writeln!(cargo_toml, "{}", Dep::try_parse(&dep)?)?;
    }

    open::open(open::OpenOpts {
        name,
        gui: opts.gui,
        no_w: opts.no_w,
        skip_check: true,
        editor_opts: opts.editor_opts,
    })
}

struct Dep<'a> {
    dep_name: &'a str,
    dep_ver: &'a str,
}

impl<'a> Dep<'a> {
    fn try_parse(dep: &'a str) -> error::Result<Self> {
        let mut parts = dep.split('=');
        let dep_name = parts.next().unwrap().trim();
        let dep_ver = parts.next().unwrap_or("*").trim();

        if parts.next().is_some() {
            return Err(error::Error::new(
                io::ErrorKind::InvalidInput,
                format!("dependency '{}' is in an incorrect format", dep),
            )
            .with_help("dependencies must either be '<dep-name>' or '<dep-name>=<dep-version>'"));
        }

        Ok(Self { dep_name, dep_ver })
    }
}

impl fmt::Display for Dep<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = \"{}\"", self.dep_name, self.dep_ver)
    }
}
