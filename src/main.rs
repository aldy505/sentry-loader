mod sentry_client;
mod sentry_dsn_builder;
mod template_files;

use crate::sentry_client::SentryClient;
use crate::sentry_dsn_builder::SentryDsnBuilder;
use crate::template_files::TemplateFiles;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::{env, fs, path};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use log::debug;

#[derive(Clone)]
struct AppState {
    pub sentry_public_hostname: String,
    pub js_sdk_version: String,
    pub use_https: bool,
    pub sentry_dsn_builder: SentryDsnBuilder,
    pub sentry_client: SentryClient,
    pub template_files: TemplateFiles,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let sentry_public_hostname =
        env::var("SENTRY_PUBLIC_HOSTNAME").unwrap_or("selfhosted.sentry.dev".to_string());
    let use_https = env::var("USE_HTTPS").unwrap_or("false".to_string()) == "true";
    let sentry_auth_token = env::var("SENTRY_AUTH_TOKEN").unwrap_or("".to_string());
    let js_sdk_version = env::var("JS_SDK_VERSION").unwrap_or("latest".to_string());
    let js_bundler_directory =
        env::var("JS_BUNDLER_DIRECTORY").unwrap_or("/var/sentry-loader/bundler".to_string());
    let template_files_base_path =
        env::var("TEMPLATE_FILES_DIRECTORY").unwrap_or("/var/sentry-loader/templates".to_string());

    // Read the js-sdk-loader.js.tmpl and js-sdk-min-loader.js.tmpl files
    let js_sdk_loader = fs::read_to_string(
        path::Path::new(&template_files_base_path.clone()).join("js-sdk-loader.js.tmpl"),
    )
    .unwrap();
    let js_sdk_min_loader = fs::read_to_string(
        path::Path::new(&template_files_base_path.clone()).join("js-sdk-loader.min.js.tmpl"),
    )
    .unwrap();

    let template_files = TemplateFiles {
        js_sdk_loader,
        js_sdk_min_loader,
    };

    let sentry_client = SentryClient::new(
        env::var("SENTRY_UPSTREAM_URL").unwrap_or("http://web:9000".to_string()),
        sentry_auth_token,
    );

    let sentry_dsn_builder = SentryDsnBuilder {
        sentry_hostname: sentry_public_hostname.clone(),
        secure: use_https,
    };

    let application_state = AppState {
        sentry_public_hostname,
        js_sdk_version: js_sdk_version.clone(),
        use_https,
        sentry_dsn_builder,
        sentry_client,
        template_files,
    };

    let app = Router::new()
        .route("/js-sdk-loader/:publickey", get(loader_with_public_key))
        .nest_service(
            format!("/{}", js_sdk_version).as_str(),
            ServeDir::new(js_bundler_directory),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(false)
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
        .with_state(application_state);

    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or("3000".to_string())
    ))
    .await
    .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn loader_with_public_key(
    State(application_state): State<AppState>,
    Path(mut publickey): Path<String>,
) -> impl IntoResponse {
    let minified_version = publickey.ends_with(".min.js");
    if publickey.ends_with(".min.js") {
        // Remove the .min.js from `publickey`
        publickey = publickey.replace(".min.js", "");
    } else if publickey.ends_with(".js") {
        publickey = publickey.replace(".js", "");
    } else {
        // Early not found return
        return (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            String::from("Not found"),
        );
    }

    debug!("Public key: {}", publickey);

        // For each organization IDs, retrieve project ID
        let project_ids = application_state
            .sentry_client
            .list_projects()
            .await
            .unwrap();

        debug!("Project IDs: {:?}", project_ids);
        for (organization_id, project_id) in project_ids.iter() {
            // For each project ID, check if there's any matching public key with the `publickey`
            let client_keys = application_state
                .sentry_client
                .list_project_client_keys(organization_id.clone(), project_id.clone())
                .await
                .unwrap();

            debug!("Client keys: {:?}", client_keys);
            for client_key in client_keys.iter() {
                if *client_key == publickey {
                    // From the found project ID, build the DSN
                    let dsn = application_state
                        .sentry_dsn_builder
                        .build_dsn(publickey.clone(), project_id.clone());
                    let protocol = match application_state.use_https {
                        true => "https",
                        false => "http",
                    };
                    let js_sdk_url = format!(
                        "{}://{}/{}/bundle.tracing.replay.min.js",
                        protocol,
                        application_state.sentry_public_hostname,
                        application_state.js_sdk_version
                    );

                    let body = match minified_version {
                        true => application_state.template_files.build_minified(
                            publickey.clone(),
                            js_sdk_url,
                            dsn,
                        ),
                        false => application_state.template_files.build(
                            publickey.clone(),
                            js_sdk_url,
                            dsn,
                        ),
                    };

                    return (StatusCode::OK, [("content-type", "text/javascript")], body.clone());
                }
            }
        }

    // Not found route
    return (
        StatusCode::NOT_FOUND,
        [("content-type", "text/plain")],
        String::from("Not found"),
    );
}
