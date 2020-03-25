extern crate serde_yaml;
extern crate structopt;

use serde::{Deserialize,Serialize};
use structopt::StructOpt;
use file_diff::diff;
use std::{io,fs};
use std::process::Command;
use std::io::Write;

// Define cli option arguments
#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    configfile: std::path::PathBuf,
}

// Define the structure of the configfile
#[derive(Deserialize, Serialize, Debug)]
struct PackageConfig {
    name: String,
    ensure: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct FileConfig {
    filename: std::path::PathBuf,
    source: std::path::PathBuf,
    owner: String,
    group: String,
    mode: String,
    trigger: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ConfigType {
    packages: Vec<PackageConfig>,
    files: Vec<FileConfig>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    // Collect the parent folder to the config file.
    // All FileConfig.source paths are relative to this folder.
    let mut config_folder = opt.configfile.clone();
    config_folder = config_folder.as_path().parent().unwrap().to_path_buf();

    // Open and read the config file (yaml or json formatted)
    let configfile = std::fs::File::open(&opt.configfile)?;
    let config: ConfigType = serde_yaml::from_reader(configfile)?;

    println!("Checking server config to match: {:?}", &opt.configfile);

    // print!("\n--- Config supplied ---\n");
    // println!("{}\n", serde_yaml::to_string(&config).unwrap());

    // Run apt-get update every time
    let apt_update_output = Command::new("apt-get")
        .arg("update")
        .output().expect("failed to restart service");
    if apt_update_output.status.success() {
        println!("+ Package List Update");
    }
    else {
        println!("+ Package List Update: - error updating package list");
        io::stderr().write_all(&apt_update_output.stderr).unwrap();
    }

    for package in config.packages.into_iter() {

        // format for apt-get install. E.g.: jq=1.4-2.1~ubuntu14.04.1 or just jq
        let mut package_version = package.name.clone();
        if package.ensure != "latest" {
            package_version = format!("{}={}", &package.name, package.ensure);
        }

        // ensure the package is installed and of the specified version
        let apt_output = Command::new("apt-get")
            .arg("install")
            .arg("-y")
            .arg("--force-yes")
            .arg(format!("{}", package_version))
            .output().expect("failed to ensure package installed");
        if apt_output.status.success() {
            println!("+ Package: {}", package.name);
        }
        else {
            println!("+ Package: {} - error installing package", package.name);
            io::stderr().write_all(&apt_output.stderr).unwrap();
        }
    }

    let mut triggers = Vec::new();
    for file in config.files.into_iter() {
        println!("+ File: {:?}", file.filename);

        // make the source path non-relative
        let file_source_path = config_folder.as_path().join(file.source.as_path());

        // check if the source and target filename match in content and existance
        let file_test = diff(file_source_path.to_str().unwrap(), file.filename.to_str().unwrap());

        // update the file content by copying it to the target filename if needed
        if ! file_test {
            println!("++ Updating file content: {:?}", file.filename);

            // check parent folder for file exists, if not, create it
            let mut file_folder = file.filename.clone();
            file_folder = file_folder.as_path().parent().unwrap().to_path_buf();
            if ! file_folder.is_dir() {
                let mkdir_output = Command::new("mkdir")
                    .arg("-p")
                    .arg(&file_folder)
                    .output().expect("failed create file parent folder");
                io::stderr().write_all(&mkdir_output.stderr).unwrap();
            }

            // Copy source to target filename
            fs::copy(file_source_path, &file.filename)?;

            // file changed, so create a trigger
            if file.trigger != "" {
                triggers.push(file.trigger)
            }
        }

        // enforce the user and group ownership on the target filename
        let chown_output = Command::new("chown")
            .arg(format!("{}:{}", file.owner, file.group))
            .arg(&file.filename)
            .output().expect("failed to set owner and group");
        io::stderr().write_all(&chown_output.stderr).unwrap();

        // enforce the file mode on the target filename
        let chmod_output = Command::new("chmod")
            .arg(file.mode)
            .arg(&file.filename)
            .output().expect("failed to set file mode");
        io::stderr().write_all(&chmod_output.stderr).unwrap();

    }

    // restart each service that has been triggered by a change
    for trigger in triggers.into_iter() {
        if trigger != "" {
            let service_output = Command::new("service")
                .arg(format!("{}", trigger))
                .arg("restart")
                .output().expect("failed to restart service");
            if service_output.status.success() {
                println!("+ Service: {}", trigger);
            }
            else {
                println!("+ Service: {} - error restarting service", trigger);
                io::stderr().write_all(&service_output.stderr).unwrap();
            }
        }
    }

    Ok(())
}
