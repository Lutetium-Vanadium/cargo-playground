use crate::{error, helpers};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CleanOpts {
    /// A regex to match against playground names. If not provided, all will be deleted.
    #[structopt(long, short)]
    matches: Option<String>,
}

pub fn clean(opts: CleanOpts) -> error::Result<()> {
    let path = helpers::get_dir();

    if !path.exists() {
        return Ok(());
    }

    let regex = match opts.matches {
        Some(matches) => {
            let regex = regex::RegexBuilder::new(&matches)
                .case_insensitive(true)
                .build()
                .map_err(|err| {
                    error::Error::new(std::io::ErrorKind::InvalidInput, err)
                })?;
            Some(regex)
        }
        None => None,
    };

    // ignoring errors for now, maybe do something about it?
    for entry in path.read_dir()?.flatten() {
        let mut path = entry.path();

        match (&regex, path.file_name().unwrap().to_str()) {
            (None, _) => {}
            (Some(regex), Some(path)) if regex.is_match(path) => {}
            _ => continue,
        }

        path.push("Cargo.toml");

        if path.exists() {
            path.pop();

            if let Err(io_err) = fs::remove_dir_all(&path) {
                let err = error::Error::new(
                    io_err.kind(),
                    format!("couldn't delete playground at {:?}: {}", path, io_err),
                )
                .with_help("check if the right directory is being cleaned");

                eprintln!("{}", err);
            }
        }
    }

    Ok(())
}
