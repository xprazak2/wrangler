mod krate;
pub mod package;
pub mod preview;
mod route;
mod upload_form;

use package::Package;
use route::Route;
use upload_form::build_script_upload_form;

use log::info;

use crate::commands;
use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn publish(user: &GlobalUser, project: &Project, release: bool) -> Result<(), failure::Error> {
    info!("release = {}", release);

    validate_project(project, release)?;
    commands::build(&project)?;
    publish_script(&user, &project, release)?;
    if release {
        info!("release mode detected, making a route...");
        let route = Route::new(&project)?;
        Route::publish(&user, &project, &route)?;
        let msg = format!(
            "Success! Your worker was successfully published. You can view it at {}.",
            &route.pattern
        );
        message::success(&msg);
    } else {
        message::success("Success! Your worker was successfully published.");
    }
    Ok(())
}

fn publish_script(
    user: &GlobalUser,
    project: &Project,
    release: bool,
) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        project.account_id, project.name,
    );

    let client = http::auth_client(user);

    let script_upload_form = build_script_upload_form(project)?;

    let mut res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    if res.status().is_success() {
        message::success("Successfully published your script.");
    } else {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    if !release {
        let private = project.private.unwrap_or(false);
        if !private {
            info!("--release not passed, publishing to subdomain");
            make_public_on_subdomain(project, user)?;
        }
    }

    Ok(())
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled":true}).to_string()
}

fn make_public_on_subdomain(project: &Project, user: &GlobalUser) -> Result<(), failure::Error> {
    info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&project.account_id, user)?;

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        project.account_id, project.name,
    );

    let client = http::auth_client(user);

    info!("Making public on subdomain...");
    let mut res = client
        .post(&sd_worker_addr)
        .header("Content-type", "application/json")
        .body(build_subdomain_request())
        .send()?;

    if res.status().is_success() {
        let msg = format!(
            "Successfully made your script available at https://{}.{}.workers.dev",
            project.name, subdomain
        );
        message::success(&msg)
    } else {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }
    Ok(())
}

fn validate_project(project: &Project, release: bool) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if project.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if project.name.is_empty() {
        missing_fields.push("name")
    };

    match &project.kv_namespaces {
        Some(kv_namespaces) => {
            for kv in kv_namespaces {
                if kv.binding.is_empty() {
                    missing_fields.push("kv-namespace binding")
                }

                if kv.id.is_empty() {
                    missing_fields.push("kv-namespace id")
                }
            }
        }
        None => {}
    }

    let destination = if release {
        //check required fields for release
        if project
            .zone_id
            .as_ref()
            .unwrap_or(&"".to_string())
            .is_empty()
        {
            missing_fields.push("zone_id")
        };
        if project.route.as_ref().unwrap_or(&"".to_string()).is_empty() {
            missing_fields.push("route")
        };
        //zoned deploy destination
        "a route"
    } else {
        //zoneless deploy destination
        "your subdomain"
    };

    let (field_pluralization, is_are) = match missing_fields.len() {
        n if n >= 2 => ("fields", "are"),
        1 => ("field", "is"),
        _ => ("", ""),
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "Your wrangler.toml is missing the {} {:?} which {} required to publish to {}!",
            field_pluralization,
            missing_fields,
            is_are,
            destination
        );
    };

    Ok(())
}
