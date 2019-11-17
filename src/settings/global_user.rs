use std::env;
use std::path::{Path, PathBuf};
use std::convert::TryFrom;

use cloudflare::framework::auth::Credentials;
use log::info;
use serde::{Deserialize, Serialize};

use crate::terminal::emoji;
use config::{Config, Environment, File, ConfigError};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalUser {
    TokenAuth { api_token: String },
    GlobalKeyAuth { email: String, api_key: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GlobalConfig {
    #[serde(flatten)]
    pub global_user: GlobalUser,
    pub next_default_project: String,
}

impl GlobalConfig {
    pub fn default_project_name() -> String {
        "worker".to_string()
    }

    pub fn new() -> Result<Self, failure::Error> {
        get_global_config()
    }
}

impl GlobalUser {
    pub fn new() -> Result<Self, failure::Error> {
        get_global_user()
    }
}

impl From<GlobalUser> for Credentials {
    fn from(user: GlobalUser) -> Credentials {
        match user {
            GlobalUser::TokenAuth { api_token } => Credentials::UserAuthToken { token: api_token },
            GlobalUser::GlobalKeyAuth { email, api_key } => Credentials::UserAuthKey {
                key: api_key,
                email,
            },
        }
    }
}

impl TryFrom<config::Config> for GlobalConfig {
    type Error = config::ConfigError;

    fn try_from(conf: config::Config) -> Result<Self, Self::Error> {

        let next_default_project = match conf.get_str("next_default_project") {
            Ok(val) => {
                if val.is_empty() {
                    GlobalConfig::default_project_name()
                } else {
                    val
                }
            },
            Err(_) => GlobalConfig::default_project_name(),
        };

        match conf.get("api_key") {
            Ok(key) => {
                let email = conf.get("email")?;
                Ok(GlobalConfig {
                    next_default_project: next_default_project,
                    global_user: GlobalUser::GlobalKeyAuth {
                        api_key: key,
                        email: email,
                    }
                })
            },
            Err(_) => {
                match conf.get("api_token") {
                    Ok(token) => {
                        Ok(GlobalConfig {
                            next_default_project: next_default_project,
                            global_user: GlobalUser::TokenAuth {
                                api_token: token
                            }
                        })
                    },
                    Err(_) => Err(ConfigError::Message("No api_key or api_token found in global config. Have you run 'wrangler config'?".to_string()))
                }
            }
        }
    }
}

fn get_global_user() -> Result<GlobalUser, failure::Error> {
    get_global_config().and_then(|config| Ok(config.global_user))
}

fn get_global_config() -> Result<GlobalConfig, failure::Error> {
    let mut s = Config::new();

    let config_path = get_global_config_dir()
        .expect("could not find global config directory")
        .join("default.toml");
    let config_str = config_path
        .to_str()
        .expect("global config path should be a string");

    // Skip reading global config if non existent
    // because envs might be provided
    if config_path.exists() {
        info!(
            "Config path exists. Reading from config file, {}",
            config_str
        );
        s.merge(File::with_name(config_str))?;
    }

    // Eg.. `CF_API_KEY=farts` would set the `account_auth_key` key
    // envs are: CF_EMAIL, CF_API_KEY and CF_API_TOKEN
    s.merge(Environment::with_prefix("CF"))?;

    let global_config: Result<GlobalConfig, config::ConfigError> = GlobalConfig::try_from(s);
    match global_config {
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

pub fn get_global_config_dir() -> Result<PathBuf, failure::Error> {
    let home_dir = if let Ok(value) = env::var("WRANGLER_HOME") {
        info!("Using WRANGLER_HOME: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        info!("No WRANGLER_HOME detected");
        dirs::home_dir()
            .expect("oops no home dir")
            .join(".wrangler")
    };
    let global_config_dir = home_dir.join("config");
    info!("Using global config dir: {:?}", global_config_dir);
    Ok(global_config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn it_can_prioritize_token_input() {
        // Set all CF_API_TOKEN, CF_EMAIL, and CF_API_KEY.
        // This test evaluates whether the GlobalUser returned is
        // a GlobalUser::TokenAuth (expected behavior; token
        // should be prioritized over email + global API key pair.)
        env::set_var("CF_API_TOKEN", "foo");
        env::set_var("CF_EMAIL", "test@cloudflare.com");
        env::set_var("CF_API_KEY", "bar");

        let user = get_global_config().unwrap();
        assert_eq!(
            user,
            GlobalUser::TokenAuth {
                api_token: "foo".to_string()
            }
        );
    }
}
