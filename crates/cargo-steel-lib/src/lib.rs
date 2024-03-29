use std::{error::Error, path::PathBuf, process::Command};

use cargo_metadata::{Message, MetadataCommand, Package};
use std::process::Stdio;

fn package_contains_dependency_on_steel(packages: &[Package]) -> Option<&Package> {
    packages.iter().find(|x| x.name == "steel-core")
}

/*
TODO:
- Desired output directory / do not copy to native automatically
- Specify target architecture
*/

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut steel_home = PathBuf::from(std::env::var("STEEL_HOME")?);

    steel_home.push("native");

    let metadata = MetadataCommand::new().exec()?;

    let package = match metadata.root_package() {
        Some(p) => p,
        None => return Err("cargo steel-lib must be run from within a crate".into()),
    };

    println!("Attempting to install: {:#?}", package.name);

    if package_contains_dependency_on_steel(&metadata.packages).is_none() {
        return Err(
            "Cannot install package as a steel dylib - does not contain a dependency on steel!"
                .into(),
        );
    }

    let mut command = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let last = cargo_metadata::Message::parse_stream(reader)
        .filter_map(|x| {
            if let Ok(Message::CompilerArtifact(artifact)) = x {
                Some(artifact)
            } else {
                None
            }
        })
        .last()
        .unwrap();

    if last.target.kind == ["cdylib"] {
        println!("Found a cdylib!");

        for file in last.filenames {
            let filename = file.file_name().unwrap();

            steel_home.push(filename);

            println!("Copying {} to {}", file, &steel_home.to_str().unwrap());

            std::fs::copy(file, &steel_home).unwrap();

            steel_home.pop();
        }
    } else if last.target.kind == ["dylib"] {
        println!("Found a dylib!");

        for file in last.filenames {
            let filename = file.file_name().unwrap();

            steel_home.push(filename);

            println!("Copying {} to {}", file, &steel_home.to_str().unwrap());

            std::fs::copy(file, &steel_home).unwrap();

            steel_home.pop();
        }
    }

    println!("Done!");

    command.wait().expect("Couldn't get cargo's exit status");

    Ok(())
}
