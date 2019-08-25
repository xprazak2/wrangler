use crate::terminal::message;
use std::fs;
use std::path::Path;

use crate::emoji;
use crate::settings::global_user::GlobalUser;

pub fn global_config(email: &str, api_key: &str) -> Result<(), failure::Error> {
    let s = GlobalUser {
        email: email.to_string(),
        api_key: api_key.to_string(),
        next_default_project: String::from("worker"),
    };

    let config_path = write_global_config(&s)?;

    message::success(&format!(
        "Successfully configured. You can find your configuration file at: {}",
        &config_path
    ));

    Ok(())
}

pub fn write_global_config(global_user: &GlobalUser) -> Result<String, failure::Error> {
    let toml = toml::to_string(global_user)?;

    let config_dir = Path::new(&dirs::home_dir().unwrap_or_else(|| {
        panic!(
            "{0} could not determine home directory. {0}",
            emoji::CONSTRUCTION
        )
    }))
    .join(".wrangler")
    .join("config");
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("default.toml");
    fs::write(&config_file, &toml)?;
    Ok(config_file.to_string_lossy().to_string())
}
