use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

enum HealthState {
    Loading,
    Ready(String),
    Failed(String),
}

impl std::fmt::Display for HealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthState::Loading => write!(f, "Loading..."),
            HealthState::Ready(text) => write!(f, "{text}"),
            HealthState::Failed(error) => write!(f, "{error}"),
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let health = use_state(|| HealthState::Loading);

    {
        let health = health.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = match Request::get("/api/health").send().await {
                    Ok(response) => response,
                    Err(error) => {
                        health.set(HealthState::Failed(format!(
                            "Connection failed: {error}"
                        )));
                        return;
                    }
                };

                if !response.ok() {
                    health.set(HealthState::Failed(format!(
                        "HTTP error: {}",
                        response.status()
                    )));
                    return;
                }

                let text = match response.text().await {
                    Ok(text) => text,
                    Err(error) => {
                        health.set(HealthState::Failed(format!(
                            "Failed to read response: {error}"
                        )));
                        return;
                    }
                };

                health.set(HealthState::Ready(text));
            })
        });
    }
    html! {
        <main>
            <h1>{"Wikipath"}</h1>
            <p>{ format!("Backend: {}", &*health) }</p>
            <p>{ "Find your way through Wikipedia" }</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
