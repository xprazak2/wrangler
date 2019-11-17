use crate::terminal::message;
use std::fs;
use std::fs::File;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::http;
use crate::settings::global_user::{get_global_config_dir, GlobalUser, GlobalConfig};

use cloudflare::endpoints::user::{GetUserDetails, GetUserTokenStatus};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

// set the permissions on the dir, we want to avoid that other user reads to file
#[cfg(not(target_os = "windows"))]
pub fn set_file_mode(file: &PathBuf) {
    File::open(&file)
        .unwrap()
        .set_permissions(PermissionsExt::from_mode(0o600))
        .expect("could not set permissions on file");
}

pub fn global_config(config: &GlobalConfig, verify: bool) -> Result<(), failure::Error> {
    if verify {
        message::info("Validating credentials...");
        validate_credentials(&config.global_user)?;
    }

    let config_path = write_global_config(&config)?;

    message::success(&format!(
        "Successfully configured. You can find your configuration file at: {}",
        &config_path
    ));

    Ok(())
}

pub fn write_global_config(global_config: &GlobalConfig) -> Result<String, ::failure::Error> {
    let toml = toml::to_string(&global_config)?;

    let config_dir = get_global_config_dir().expect("could not find global config directory");
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("default.toml");
    fs::write(&config_file, &toml)?;

    // set permissions on the file
    #[cfg(not(target_os = "windows"))]
    set_file_mode(&config_file);

    Ok(config_file.to_string_lossy().to_string())
}

// validate_credentials() checks the /user/tokens/verify endpoint (for API token)
// or /user endpoint (for global API key) to ensure provided credentials actually work.
pub fn validate_credentials(user: &GlobalUser) -> Result<(), failure::Error> {
    let client = http::api_client(user, HttpApiClientConfig::default())?;

    match user {
        GlobalUser::TokenAuth { .. } => {
            match client.request(&GetUserTokenStatus {}) {
                Ok(success) => {
                    if success.result.status == "active" {
                        Ok(())
                    } else {
                        failure::bail!("Authentication check failed. Your token has status \"{}\", not \"active\".\nTry rolling your token on the Cloudflare dashboard.")
                    }
                },
                Err(e) => failure::bail!("Authentication check failed. Please make sure your API token is correct.\n{}", http::format_error(e, None))
            }
        }
        GlobalUser::GlobalKeyAuth { .. } => {
            match client.request(&GetUserDetails {}) {
                Ok(_) => Ok(()),
                Err(e) => failure::bail!("Authentication check failed. Please make sure your email and global API key pair are correct. (https://developers.cloudflare.com/workers/quickstart/#global-api-key)\n{}", http::format_error(e, None)),
            }
        }
    }
}
