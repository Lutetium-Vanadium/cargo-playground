use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, io};

pub struct Examples {
    pub path: PathBuf,
    pub examples: Vec<String>,
}

impl Examples {
    pub fn find(manifest: &mut PathBuf, dep_name: &str) -> io::Result<Option<Examples>> {
        // make sure that the template dependency is present
        if !Command::new("cargo")
            .arg("fetch")
            .arg("--manifest-path")
            .arg(&manifest)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success()
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not fetch template dependency",
            ));
        }

        manifest.set_extension("lock");
        let lock_file = fs::File::open(&manifest)?;
        manifest.set_extension("toml");

        let mut lines = io::BufReader::new(lock_file).lines();

        let mut dep = String::with_capacity(dep_name.len() + 2);
        dep.push('"');
        dep.push_str(dep_name);
        dep.push('"');

        loop {
            let line = lines
                .next()
                .expect("didn't find entry for target package")?;
            if line.starts_with("name") && line.contains(&dep) {
                break;
            }
        }

        let dep_path = {
            let line = lines.next().unwrap()?;

            assert!(line.starts_with("version"));

            let start = line.find('"').unwrap() + 1;
            let end = line.rfind('"').unwrap();

            let mut s = String::with_capacity(dep_name.len() + 1 + end - start);

            s += dep_name;
            s.push('-');
            s += &line[start..end];

            s
        };

        let mut examples_path = get_cargo_src_root(&dep_path).map_err(|err| {
            io::Error::new(err.kind(), format!("couldn't find cargo src root: {}", err))
        })?;

        examples_path.push("examples");

        if !examples_path.is_dir() {
            return Ok(None);
        }

        let mut examples = Vec::new();

        for example in examples_path.read_dir()?.filter_map(Result::ok) {
            if let Ok(example) = example.file_name().into_string() {
                if example.ends_with(".rs") {
                    examples.push(example.to_owned());
                }
            }
        }

        if examples.is_empty() {
            return Ok(None);
        }

        Ok(Some(Examples {
            path: examples_path,
            examples,
        }))
    }
}

fn get_cargo_src_root(dep_path: &str) -> io::Result<PathBuf> {
    let mut cargo_home = home::cargo_home()?;

    cargo_home.push("registry");
    cargo_home.push("src");

    for dir in cargo_home.read_dir()?.flat_map(Result::ok) {
        let mut path = dir.path();
        path.push(dep_path);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        cargo_home.to_str().unwrap_or("src is empty"),
    ))
}
