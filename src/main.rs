#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use ferris_the_waiter::app::*;
    use ferris_the_waiter::fileserv::file_and_error_handler;
    use ferris_the_waiter::{Commands, PasswordHashString, RateLimiting};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::sync::{Arc, Mutex};

    #[derive(Serialize, Deserialize, Debug)]
    struct Config {
        commands: Commands,
        password_hash: String,
    }

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let mut config_file = File::open(
        env::var("FERRIS_WAITER_CONFIG")
            .expect("config path to be configured with 'FERRIS_WAITER_CONFIG'"),
    )?;

    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;

    let commands_resource = Arc::new(config.commands);
    let password_hash_resource = Arc::new(PasswordHashString(config.password_hash));

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options)
        .layer(axum::Extension(commands_resource))
        .layer(axum::Extension(password_hash_resource))
        .layer(axum::Extension(Arc::new(Mutex::new(
            RateLimiting::default(),
        ))));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
