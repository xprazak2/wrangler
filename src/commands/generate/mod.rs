use std::path::PathBuf;
use std::process::Command;

use crate::commands::validate_worker_name;
use crate::settings::target::{Manifest, Site, TargetType};
use crate::settings::global_user::GlobalConfig;
use crate::{commands, install};

pub fn generate(
    name: &str,
    template: &str,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    validate_worker_name(name)?;

    log::info!("Generating a new worker project with name '{}'", name);
    run_generate(name, template)?;

    let config_path = PathBuf::from("./").join(&name);
    // TODO: this is tightly coupled to our site template. Need to remove once
    // we refine our generate logic.
    let generated_site = if site {
        Some(Site::new("./public"))
    } else {
        None
    };
    Manifest::generate(name.to_string(), target_type, &config_path, generated_site)?;

    Ok(())
}

pub fn run_generate(name: &str, template: &str) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams")?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let command = command(binary_path, &args);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)
}

pub fn write_project_name(global_config: &GlobalConfig) -> Result<(), failure::Error> {
    let new_project = generate_project_name(global_config);

    let mut new_config = global_config.clone();
    new_config.next_default_project = new_project;
    commands::config::write_global_config(&new_config)?;
    Ok(())
}

fn generate_project_name(global_config: &GlobalConfig) -> String {
    let current_project = &global_config.next_default_project;
    let mut new_project: Vec<&str> = current_project.split("-").collect();

    let last_item = new_project[new_project.len() - 1];

    let next_num = match last_item.parse::<i32>() {
        Ok(val) => {
            new_project.pop();
            val + 1
        }
        Err(_) => 1
    };

    let mut new_project: Vec<String> = new_project.into_iter().map(|item| item.to_string()).collect();
    new_project.push(next_num.to_string());
    new_project.join("-")
}

fn command(binary_path: PathBuf, args: &[&str]) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}
