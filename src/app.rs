use leptos::ev::SubmitEvent;
use leptos::html::{Input, Select};
use leptos::*;
use leptos_meta::*;

#[server(CommandList, "/ferris/api")]
pub async fn command_list() -> Result<Vec<String>, ServerFnError> {
    use crate::Commands;
    use axum::extract::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    let Extension(commands): Extension<Arc<Commands>> = extract().await?;

    Ok(commands.0.keys().cloned().collect())
}

#[server(InvokeCommand, "/ferris/api")]
pub async fn invoke_command(command_name: String, password: String) -> Result<(), ServerFnError> {
    use crate::Commands;
    use crate::PasswordHashString;
    use crate::RateLimiting;
    use axum::extract::Extension;
    use leptos_axum::extract;
    use std::sync::{Arc, Mutex};
    use tokio::process::Command;

    let Extension(commands): Extension<Arc<Commands>> = extract().await?;
    let Extension(password_hash): Extension<Arc<PasswordHashString>> = extract().await?;
    let Extension(rate_limiting): Extension<Arc<Mutex<RateLimiting>>> = extract().await?;

    {
        let rate_limiting = rate_limiting.lock().unwrap();

        let now = std::time::Instant::now();

        if let Some(last_request_time) = (*rate_limiting).last_request_time {
            if now.duration_since(last_request_time) < std::time::Duration::from_millis(5000) {
                return Err(ServerFnError::ServerError("Rate limited".to_string()));
            }
        }
    }

    if !bcrypt::verify(password.as_bytes(), password_hash.0.as_str())? {
        {
            let mut rate_limiting = rate_limiting.lock().unwrap();

            let now = std::time::Instant::now();
            rate_limiting.last_request_time = Some(now);
        }
        return Err(ServerFnError::ServerError("Invalid password".to_string()));
    }
    {
        let mut rate_limiting = rate_limiting.lock().unwrap();

        let now = std::time::Instant::now();
        rate_limiting.last_request_time = Some(now);
    }

    match commands.0.get(&command_name) {
        None => Err(ServerFnError::ServerError(
            "No command with that name".to_string(),
        )),
        Some(command_str) => {
            let mut command = Command::new("sh");
            command
                .arg("-c")
                .args(shlex::Shlex::new(format!("\"{command_str}\"").as_str()));
            command.spawn()?.wait().await?;

            Ok(())
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let commands = create_resource(|| (), |_| async move { command_list().await });

    let command_input: NodeRef<Select> = create_node_ref();
    let password_input: NodeRef<Input> = create_node_ref();

    let submit_action = create_action(|(command_name, password): &(String, String)| {
        let command_name = command_name.to_owned();
        let password = password.to_owned();
        async move { invoke_command(command_name, password).await }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let command_name = command_input().expect("<input> to exist").value();
        let password = password_input().expect("<input> to exist").value();

        submit_action.dispatch((command_name, password));
    };

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/ferris-the-waiter.css"/>
        <h1>Let Ferris Help You Out!</h1>
        <svg width="120" height="120">
            <image xlink:href="https://rustacean.net/assets/cuddlyferris.svg" src="https://rustacean.net/assets/cuddlyferris.svg" width="120" height="120"/>
        </svg>

        {
            move || {
                match submit_action.value()() {
                    Some(Err(error)) => {
                        let error = match error {
                            ServerFnError::ServerError(error) => error,
                            _ => format!("{error}"),
                        };

                        (view! {
                            <div class="error">{error.to_string()}</div>
                        }).into_view()
                    },
                    Some(Ok(_)) => {
                         (view! {
                            <div class="success">Success!</div>
                        }).into_view()
                    },
                    _ => (view! {}).into_view()
                }
            }
        }

        <form on:submit=on_submit>
            <Suspense fallback=move || view! { <select>LOADING</select> }>
            <div class="input">
                <label for="command">Command</label>
                <select id="command" node_ref=command_input>{move || {
                    match commands() {
                        Some(Ok(commands)) => {
                            commands.iter().map(|command| {
                                view! {
                                    <option value=command>{command}</option>
                                }
                            }).collect::<Vec<_>>().into_view()
                        },
                        Some(Err(_)) => (view! {}).into_view(),
                        None => (view! {}).into_view(),
                    }
                }
                }
                </select>
            </div>
            </Suspense>
            <div class="input">
                <label for="password">Password</label>
                <input id="password" type="password" node_ref=password_input></input>
            </div>
            <input type="submit" value="Pretty please, Ferris!"></input>
        </form>
    }
}
