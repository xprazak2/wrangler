use crate::settings::project::{Project, ProjectType};
use crate::settings::global_user::GlobalUser;
use crate::{commands, install};
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::{emoji, message};

pub fn generate(name: &str, template: &str, pt: Option<ProjectType>) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams")?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let pt = pt.unwrap_or_else(|| project_type(template));
    let command = command(name, binary_path, &args, &pt);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Project::generate(name.to_string(), pt, false)?;
    Ok(())
}

pub fn write_project_name(global_user: &GlobalUser) -> Result<(), failure::Error> {
    let new_project = generate_project_name(global_user);

    let mut new_user = global_user.clone();
    new_user.next_default_project = new_project;
    commands::config::write_global_config(&new_user)?;
    Ok(())
}

fn generate_project_name(global_user: &GlobalUser) -> String {
    let current_project = &global_user.next_default_project;
    let mut new_project: Vec<&str> = current_project.split("-").collect();

    let last = new_project[new_project.len() - 1];

    let next_num = match last.parse::<i32>() {
        Ok(val) => {
            new_project.pop();
            val + 1
        }
        Err(_) => {
            1
        }
    };

    let mut new_project: Vec<String> = new_project.into_iter().map(|item| item.to_string()).collect();
    new_project.push(next_num.to_string());
    new_project.join("-")
}

fn command(name: &str, binary_path: PathBuf, args: &[&str], project_type: &ProjectType) -> Command {
    let msg = format!(
        "{} Generating a new {} worker project with name '{}'...",
        emoji::SHEEP,
        project_type,
        name
    );

    message::working(&msg);

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

fn project_type(template: &str) -> ProjectType {
    if template.contains("rust") {
        return ProjectType::Rust;
    }
    ProjectType::default()
}
