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
    #[structopt(short, long, env = "VISUAL", hide_env_values = true)]
    /// The editor to open the project in
    editor: String,
    #[structopt(short, long)]
    /// Extra args (if any) to be supplied to the editor
    args: Vec<String>,
    /// The dependencies to add. It must be in the following format:
    /// 1. <dep-name>
    /// 2. <dep-name>=<dep-version>
    #[structopt(name = "dependencies")]
    deps: Vec<String>,
}

pub fn create(opts: NewOpts) -> io::Result<()> {
    let name = opts.name.unwrap_or_else(|| {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Current time is unix epoch!");
        time.as_secs().to_string()
    });

    println!("Creating new project: {}", name);

    let mut path = super::get_dir();
    path.push(&name);

    if !Command::new("cargo")
        .arg("new")
        .arg(&path)
        .status()?
        .success()
    {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Could not create cargo project",
        ));
    }

    path.push("Cargo.toml");
    let mut cargo_toml = fs::OpenOptions::new().append(true).open(&path)?;
    cargo_toml.write_all(b"\n")?;
    for dep in opts.deps {
        let mut parts = dep.split('=');
        let dep_name = parts.next().unwrap();
        let dep_ver = parts.next().unwrap_or("*");

        // FIXME: Deal with this better
        assert!(parts.next().is_none(), "Invalid dependency");

        writeln!(cargo_toml, "{} = \"{}\"", dep_name, dep_ver)?;
    }

    crate::open::open(crate::open::OpenOpts {
        editor: opts.editor,
        args: opts.args,
        name,
    })
}
