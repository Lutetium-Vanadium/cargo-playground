use crate::{error, open};
use std::io::Write;
use std::process::Command;
use std::time::SystemTime;
use std::{fs, io};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct NewOpts {
    #[structopt(short, long)]
    /// The name of the playground to create. If not supplied, the current timestamp will be used
    name: Option<String>,
    #[structopt(flatten)]
    editor_opts: super::EditorOpts,
    #[structopt(short, long)]
    /// Force start the playground
    force: bool,
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

    println!("creating new project: {}", name);

    let mut path = super::get_dir();
    path.push(&name); // Now represents the playground directory

    if !Command::new("cargo")
        .arg("new")
        .arg(&path)
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
    for dep in opts.deps {
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

        writeln!(cargo_toml, "{} = \"{}\"", dep_name, dep_ver)?;
    }

    open::open(open::OpenOpts {
        name,
        // FIXME: we create a project without even checking if we can open it
        force: opts.force,
        skip_check: true,
        editor_opts: opts.editor_opts,
    })
}
