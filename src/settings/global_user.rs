use crate::terminal::emoji;

use std::convert::TryFrom;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalUser {
    pub email: String,
    pub api_key: String,
    pub next_default_project: String,
}

impl GlobalUser {
    pub fn new() -> Result<Self, failure::Error> {
        get_global_config()
    }
}

impl TryFrom<config::Config> for GlobalUser {
    type Error = config::ConfigError;

    fn try_from(conf: config::Config) -> Result<Self, Self::Error> {
        let email = conf.get("email")?;

        let api_key = conf.get("api_key")?;

        let project_default_name = "worker".to_string();

        let next_default_project = match conf.get_str("next_default_project") {
            Ok(val) => {
                if val.is_empty() {
                    project_default_name
                } else {
                    val
                }
            },
            Err(_) => project_default_name,
        };

        Ok(GlobalUser {
            email,
            api_key,
            next_default_project,
        })
    }
}

fn get_global_config() -> Result<GlobalUser, failure::Error> {
    let mut s = Config::new();

    let config_path = dirs::home_dir()
        .expect("oops no home dir")
        .join(".wrangler/config/default");
    let config_str = config_path
        .to_str()
        .expect("global config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    let global_user: Result<GlobalUser, config::ConfigError> = GlobalUser::try_from(s);

    match global_user {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "{} Your global config has an error, run `wrangler config`: {}",
                emoji::WARN,
                e
            );
            failure::bail!(msg)
        }
    }
}
